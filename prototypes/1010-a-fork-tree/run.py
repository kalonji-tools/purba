"""Run the tests in files or folders, and print what each case shows.

Prototype of #1010 Q1, on the prototype of #980. Never merge.

#1010 Q1 asks whether a chain runs as a fork tree. `--tree` runs it so: each stage's act runs
once, then the stage forks once for its own checks and once for each later stage. Without
`--tree` the runner replays, as #966 decided: each later case runs every earlier case of its
chain again first. A thread alive after the act, a stock that the chain reads, or a file,
socket or pipe that the act left open makes the fork tree fall back to replay. In both modes a
refused earlier stage blocks each later stage.

    python run.py [--tree] [path ...]

#980 moves rules from the code to the run: the act's paths, an assert in the act, a line after an expected raise, a
check that changed what a later check reads, a stock that a check changed, and a case with no check are seen while
the case runs. Case names are read from the module after import. `FORM_PATHS=off` and `FORM_ALONE=off` exist only to
measure what the watch and the forks cost.

#979 adds: any `with` in the act is a setup; the act ends at the first check, so checks sit at the end of a `with`,
after it, or both; a setup that swallows a raise and skips a line fails the case, and so does an undo that raises;
the act may not replace a hook the runner sees faults through; the parent prints through a stream of its own.

    python run.py [path ...]     each path a file or a folder; the default is the current folder

A test is a function named `test…` in a file named `test_….py`. The runner reads its source, refuses what breaks the
form, and imports the file with each test's last asserts rewritten as checks, as pytest rewrites an assert.
Then it runs each test as written, once for each case, in a process of its own; the checks read in order, and each
later check is also read alone, in a fork taken before any check read (#980). The exit status is 0 when every case passes, and 1 when a case fails, is refused or is blocked.
"""

from __future__ import annotations

import ast
import builtins
import difflib
import dis
import importlib.util
import inspect
import json
import math
import mmap
import operator
import os
import shutil
import signal
import sys
import sysconfig
import tempfile
import threading
import traceback
import types
import typing
import warnings
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from cases import BEFORE, CREATED, OWNED, RAISED, SEEN, Case, Checkpoints, Fingerprints, Missed, Samples, Step, Stock, Use  # noqa: E402

SECONDS = 30
LONG = 400
HOOK = "__form_check__"
OUT = os.fdopen(os.dup(1), "w", buffering=1)  # the runner's own stream: a test that replaces sys.stdout cannot mute it (#979)


def say(text: str) -> None:
    OUT.write(text + "\n")
    OUT.flush()


def flush() -> None:
    """Flush what the test may have left in sys.stdout and sys.stderr, which it may also have replaced, or set to None."""
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.flush()
        except Exception:
            pass


# ── ① PARSE: read each test, and refuse before anything runs ────────────────────────

LOGIC: dict[type, str] = {  # #980: the paths of the act (if, loops, try, raise, assert) are seen while the case runs
    ast.With: "a `with` inside a loop or another block: a setup wraps statements of the act itself",
    ast.AsyncWith: "an `async with`",
    ast.Global: "a `global`: take the value as an input", ast.Nonlocal: "a `nonlocal`: take the value as an input",
    ast.Yield: "a `yield`: a test returns its product", ast.YieldFrom: "a `yield`: a test returns its product",
    ast.Await: "an `await`: this form runs no coroutine",
}
EXPECTERS = {"raises", "warns", "handling"}
OPS: dict[type, tuple[str, Callable[[Any, Any], bool]]] = {
    ast.Eq: ("==", operator.eq), ast.NotEq: ("!=", operator.ne), ast.Lt: ("<", operator.lt), ast.LtE: ("<=", operator.le),
    ast.Gt: (">", operator.gt), ast.GtE: (">=", operator.ge), ast.In: ("in", lambda a, b: a in b),
    ast.NotIn: ("not in", lambda a, b: a not in b), ast.Is: ("is", operator.is_), ast.IsNot: ("is not", operator.is_not),
}


@dataclass
class Check:
    """One of a test's last asserts: one comparison, with the reading on one side and the expected value on the other."""

    node: ast.Assert
    op: type
    reading_left: bool
    names: set[str]  # the names the reading reads

    @property
    def line(self) -> int:
        return self.node.lineno

    @property
    def label(self) -> str:
        text = ast.unparse(self.node.test)
        return text if len(text) <= 60 else text[:59] + "…"


@dataclass
class Shape:
    """A test, read from its source: the act, the checks, and what refuses it."""

    fn: types.FunctionType
    node: ast.FunctionDef
    file: str
    checks: list[Check] = field(default_factory=list)
    returns: ast.expr | None = None
    expects: list[str] = field(default_factory=list)  # `raises`, `warns` and `handling`, as written
    expected_inputs: set[str] = field(default_factory=set)  # inputs that a case fills with an expected value
    made_in_raises: set[str] = field(default_factory=set)
    raises_spans: list[tuple[int, int]] = field(default_factory=list)  # the lines of each `with raises(…):` (#980)
    faults: list[str] = field(default_factory=list)

    @property
    def name(self) -> str:
        return self.fn.__name__

    @property
    def params(self) -> dict[str, inspect.Parameter]:
        return dict(inspect.signature(self.fn).parameters)


@dataclass
class Source:
    path: Path
    tree: ast.Module
    bindings: dict[str, list[int]] = field(default_factory=dict)
    values: dict[str, ast.expr] = field(default_factory=dict)  # each name bound once, at module level, to its value

    @classmethod
    def read(cls, path: Path) -> Source:
        tree = ast.parse(path.read_text(), str(path))
        src = cls(path, tree)
        for stmt in tree.body:
            targets: list[ast.expr] = []
            value: ast.expr | None = None
            if isinstance(stmt, ast.Assign):
                targets, value = stmt.targets, stmt.value
            elif isinstance(stmt, ast.AnnAssign):
                targets, value = list[ast.expr]([stmt.target]), stmt.value
            for target in targets:
                if isinstance(target, ast.Name):
                    src.bindings.setdefault(target.id, []).append(stmt.lineno)
                    if value is not None:
                        src.values[target.id] = value
            if isinstance(stmt, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                src.bindings.setdefault(stmt.name, []).append(stmt.lineno)
        src.values = {k: v for k, v in src.values.items() if len(src.bindings[k]) == 1}
        return src

    def statement_at(self, line: int) -> ast.stmt | None:
        return next((s for s in self.tree.body if s.lineno <= line <= (s.end_lineno or s.lineno)), None)

    def function(self, name: str) -> ast.FunctionDef | ast.AsyncFunctionDef | None:
        defs = [s for s in self.tree.body if isinstance(s, (ast.FunctionDef, ast.AsyncFunctionDef)) and s.name == name]
        return defs[-1] if defs else None  # Python keeps the last `def` of a name


def split(node: ast.FunctionDef) -> tuple[list[ast.stmt], list[ast.Assert], ast.Return | None]:
    """A test's body: the act, its checks, and a last `return`. The act may end inside a `with`, whose last asserts are
    checks too, run before the asserts after it (#979). The act keeps that `with`; its checks are listed apart."""
    body = node.body
    if body and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant) and isinstance(body[0].value.value, str):
        body = body[1:]
    ret = body[-1] if body and isinstance(body[-1], ast.Return) else None
    if ret is not None:
        body = body[:-1]
    act, checks = tail(body)
    return act, checks, ret


def is_setup(stmt: ast.stmt) -> bool:
    """A `with` that sets something up for the statements inside it: any `with` but `raises`, `warns` and `handling`."""
    return isinstance(stmt, ast.With) and not any(expecter(item) for item in stmt.items)


def tail(body: list[ast.stmt]) -> tuple[list[ast.stmt], list[ast.Assert]]:
    """A run of statements: its act, and its checks, the asserts at its end, after those at the end of its last `with`."""
    cut = len(body)
    while cut > 0 and isinstance(body[cut - 1], ast.Assert):
        cut -= 1
    checks = [s for s in body[cut:] if isinstance(s, ast.Assert)]
    act = body[:cut]
    if act and is_setup(act[-1]):
        checks = tail(act[-1].body)[1] + checks
    return act, checks


def name_of(fn: object) -> str:
    return str(getattr(fn, "__name__", fn))


def hint(word: str, choices: list[str]) -> str:
    close = difflib.get_close_matches(word, choices, n=1)
    return f"; did you mean {close[0]!r}?" if close else f"; it has {', '.join(map(repr, choices)) or 'none'}"


def literal_or_why(node: ast.expr, src: Source) -> str | None:
    if isinstance(node, ast.Constant):
        return None
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, (ast.USub, ast.UAdd)) and isinstance(node.operand, ast.Constant):
        return None
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        return next((w for w in (literal_or_why(e, src) for e in node.elts) if w), None)
    if isinstance(node, ast.Dict):
        return next((w for w in (literal_or_why(e, src) for e in [*node.keys, *node.values] if e is not None) if w), None)
    if isinstance(node, ast.Name) and node.id in src.values:  # a name passes when its one binding is a literal
        return literal_or_why(src.values[node.id], src)
    if isinstance(node, ast.BinOp):  # #980: operators on literals, such as `"7" * 20 + "\n"`, compute nothing of the code's
        return literal_or_why(node.left, src) or literal_or_why(node.right, src)
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.Invert):
        return literal_or_why(node.operand, src)
    return f"computes `{ast.unparse(node)}`; an expected value is a literal, or operators on literals, so the test cannot compute what the code computes"


def logic_in(node: ast.AST) -> list[str]:
    """Each statement that this form does not take, read from code. The paths of the act, and a function that the test
    calls or hands to the code, are seen while the case runs (#980), so the walk does not enter a nested function or class."""
    faults: list[str] = []

    def visit(n: ast.AST) -> None:
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda, ast.ClassDef)):
            return
        if type(n) in LOGIC:
            faults.append(f"line {getattr(n, 'lineno', '?')} holds {LOGIC[type(n)]}")
        for child in ast.iter_child_nodes(n):
            visit(child)

    visit(node)
    return faults


def expected_or_why(node: ast.expr, shape: Shape, src: Source) -> str | None:
    """An expected value is a literal, an input of the test, or a list, tuple, set or dict of these."""
    if isinstance(node, ast.Name) and node.id in shape.params:
        return None
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        return next((w for w in (expected_or_why(e, shape, src) for e in node.elts) if w), None)
    if isinstance(node, ast.Dict):
        return next((w for w in (expected_or_why(e, shape, src) for e in [*node.keys, *node.values] if e is not None) if w), None)
    return literal_or_why(node, src)


def inputs_named(node: ast.expr, shape: Shape) -> set[str]:
    return {n.id for n in ast.walk(node) if isinstance(n, ast.Name) and n.id in shape.params}


def stored(nodes: list[ast.stmt]) -> set[str]:
    module = ast.Module(body=nodes, type_ignores=[])
    return {n.id for n in ast.walk(module) if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Store)}


def expecter(item: ast.withitem) -> str | None:
    call = item.context_expr
    if isinstance(call, ast.Call):
        fn = call.func
        name = fn.id if isinstance(fn, ast.Name) else fn.attr if isinstance(fn, ast.Attribute) else None
        if name in EXPECTERS:
            return name
    return None


def read_with(stmt: ast.With, shape: Shape, src: Source, walk: Callable[[list[ast.stmt]], None]) -> None:
    if is_setup(stmt):  # a setup: what it sets up holds for the statements inside it, and ends with them (#979)
        for item in stmt.items:
            shape.faults += logic_in(item.context_expr)
        walk(stmt.body)
        return
    kind = expecter(stmt.items[0]) if len(stmt.items) == 1 else None
    if kind is None:
        shape.faults.append(f"line {stmt.lineno}: `raises`, `warns` and `handling` each take a `with` of their own")
        return
    if len(stmt.body) != 1:
        shape.faults.append(f"line {stmt.lineno}: `with {kind}(…):` holds one statement, the one that raises or warns; it holds {len(stmt.body)}")
    if kind == "raises":  # #980: a line of the act that runs after it is seen while the case runs
        shape.raises_spans.append((stmt.lineno, stmt.end_lineno or stmt.lineno))
    if kind == "handling" and stmt.items[0].optional_vars is not None:
        shape.faults.append(f"line {stmt.lineno}: `with handling(…) as …`: it gives nothing; drop the `as`")
    call = stmt.items[0].context_expr
    assert isinstance(call, ast.Call)
    for arg in [*call.args[1:], *(k.value for k in call.keywords)]:
        if kind != "handling" and not (isinstance(arg, ast.Name) and arg.id in shape.params) and (why := literal_or_why(arg, src)) is not None:
            shape.faults.append(f"line {stmt.lineno}: `{kind}(…)` {why}")
        shape.expected_inputs |= inputs_named(arg, shape) if kind != "handling" else set()
    shape.expects.append(ast.unparse(call))
    for inner in stmt.body:
        shape.faults += logic_in(inner)
    if kind == "raises":
        shape.made_in_raises |= stored(stmt.body)


def read_check(stmt: ast.Assert, shape: Shape, src: Source) -> Check | None:
    test = stmt.test
    where = f"line {stmt.lineno}"
    if not isinstance(test, ast.Compare) or len(test.ops) != 1 or type(test.ops[0]) not in OPS:
        shape.faults.append(f"{where}: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`")
        return None
    left, right = test.left, test.comparators[0]
    reading_left = expected_or_why(right, shape, src) is None or expected_or_why(left, shape, src) is not None
    reading, expected = (left, right) if reading_left else (right, left)
    if (why := expected_or_why(expected, shape, src)) is not None:
        shape.faults.append(f"{where}: the expected value {why}; write a literal, or take it as an input of the case")
    shape.expected_inputs |= inputs_named(expected, shape)
    for fault in logic_in(reading):
        shape.faults.append(f"{where}: the reading {fault.split(' holds ', 1)[1]}" if " holds " in fault else fault)
    names = {n.id for n in ast.walk(reading) if isinstance(n, ast.Name)}
    if not names:
        shape.faults.append(f"{where}: `{ast.unparse(reading)}` reads no value; read what the act made")
    message = stmt.msg
    if message is not None and not (isinstance(message, ast.JoinedStr) or (isinstance(message, ast.Constant) and isinstance(message.value, str))):
        shape.faults.append(f"{where}: the message of a check is a string, or an f-string")
    late = sorted(names & shape.made_in_raises)
    if late:
        shape.faults.append(f"{where} reads {', '.join(late)}, which the act assigns inside `with raises(…):`, so it never exists; read `caught.value` with `with raises(…) as caught:`")
    return Check(stmt, type(test.ops[0]), reading_left, names)


def shape_of(fn: types.FunctionType, src: Source) -> Shape:
    node = src.function(fn.__name__)
    if not isinstance(node, ast.FunctionDef):
        stub = ast.parse(f"def {fn.__name__}(): pass").body[0]
        assert isinstance(stub, ast.FunctionDef)
        stub.lineno = node.lineno if node else fn.__code__.co_firstlineno
        shape = Shape(fn, stub, str(src.path))
        shape.faults.append("an async test: this form runs no coroutine" if node else "its source is not a `def` of this file")
        return shape
    shape = Shape(fn, node, str(src.path))
    if node.decorator_list:
        shape.faults.append(f"line {node.lineno}: a test takes no decorator; give its cases with `name = case({fn.__name__}, …)`")
    if node.args.vararg or node.args.kwarg:
        shape.faults.append(f"line {node.lineno}: `*args` or `**kwargs`: name each input")
    act, asserts, ret = split(node)
    is_check = {id(a) for a in asserts}

    def walk(stmts: list[ast.stmt]) -> None:
        stmts = [s for s in stmts if id(s) not in is_check]
        for stmt in stmts:
            if isinstance(stmt, ast.Return):
                shape.faults.append(f"line {stmt.lineno} returns before the checks: `return` is the last line of a test")
            elif isinstance(stmt, ast.With):
                read_with(stmt, shape, src, walk)
            else:
                shape.faults += logic_in(stmt)

    walk(act)
    for stmt in asserts:
        if check := read_check(stmt, shape, src):
            shape.checks.append(check)
    shape.returns = ret.value if ret else None
    if shape.returns is not None:
        shape.faults += logic_in(shape.returns)
        if any(e.startswith("raises") for e in shape.expects):
            shape.faults.append(f"line {shape.returns.lineno}: a test that expects a raise makes no product, so it returns nothing")
    # #980: "checks nothing" is seen while the case runs, when no check ran
    for name, p in shape.params.items():
        if isinstance(p.default, Use) and isinstance(p.default.factory, type) and issubclass(p.default.factory, Stock):
            stock = p.default.factory
            if inspect.isabstract(stock):
                missing = ", ".join(sorted(getattr(stock, "__abstractmethods__", ())))
                shape.faults.append(f"the input {name}: the stock {stock.__name__} lacks {missing}; a misspelled method leaves it abstract")
                continue
            if not issubclass(stock, (Samples, Checkpoints)):
                shape.faults.append(f"the input {name}: the stock {stock.__name__} offers neither Samples nor Checkpoints, so nothing seals it for the case")
            if not issubclass(stock, (Fingerprints, Checkpoints)):  # #980: by its type, not by the names a check reads
                shape.faults.append(f"the input {name}: the stock {stock.__name__} offers neither Fingerprints nor Checkpoints, so a check that changes it cannot be seen")
    return shape


IMMUTABLE = (str, bytes, int, float, bool, type(None), tuple)


def annotation_types(annotation: Any, scope: dict[str, Any]) -> tuple[type, ...] | None:
    """The run-time types an annotation allows, or None when it is not read (a name only a stub declares, Any, object)."""
    if annotation is inspect.Parameter.empty:
        return None
    if isinstance(annotation, str):
        try:
            annotation = eval(annotation, scope)  # a name that exists only for the type checker stays unread
        except Exception:
            return None
    if annotation is None:
        return (type(None),)
    if isinstance(annotation, types.UnionType) or getattr(annotation, "__origin__", None) is typing.Union:
        parts = [annotation_types(a, scope) for a in annotation.__args__]
        return None if any(p is None for p in parts) else tuple(t for p in parts if p for t in p)
    origin = getattr(annotation, "__origin__", None)
    if isinstance(origin, type):
        return (origin,)
    if isinstance(annotation, type) and annotation is not object:
        return (annotation,)
    return None


def fits(value: object, allowed: tuple[type, ...]) -> bool:
    return any(isinstance(value, k) and not (k is int and type(value) is bool) for k in allowed)


def case_call(case: Case[Any], src: Source) -> ast.Call | None:
    stmt = src.statement_at(case.line)
    for node in ast.walk(stmt) if stmt else []:
        if isinstance(node, ast.Call) and node.args and isinstance(node.args[0], ast.Name) and node.args[0].id == name_of(case.test):
            return node
    return None


def refuse_case(case: Case[Any], shape: Shape, src: Source, scope: dict[str, Any], shapes: dict[Any, Shape], written: bool = True) -> list[str]:
    """Every fault of one case, all at once."""
    faults: list[str] = []
    params = shape.params
    if case.args:
        faults.append(f"passes {len(case.args)} input(s) by position; name each input, as `case({shape.name}, name=…)`")
    for key in case.inputs:
        if key not in params:
            faults.append(f"gives {key!r}, which is no input of {shape.name}{hint(key, list(params))}")
    for name, p in params.items():
        if name not in case.inputs and p.default is inspect.Parameter.empty:
            faults.append(f"gives no {name!r}, an input of {shape.name} with no default")
    call = case_call(case, src)
    if call is None and written:
        faults.append(f"is not written `case({shape.name}, …)`; call `case` with the test's own name, so the runner can read the expected values")
    elif call is not None and any(kw.arg is None for kw in call.keywords):
        faults.append("passes inputs with `**`; name each input, as `case(…, name=…)`")
    for kw in call.keywords if call else []:
        if kw.arg in shape.expected_inputs and (why := literal_or_why(kw.value, src)) is not None:
            faults.append(f"gives {kw.arg}=…, an expected value, which {why}")
    for key, value in case.inputs.items():
        if key in params and not isinstance(value, (Step, Use)):
            allowed = annotation_types(params[key].annotation, scope)
            if allowed and not fits(value, allowed):
                faults.append(f"gives {key}={short(value)} ({kind(value)}), and {shape.name} declares {' | '.join(t.__name__ for t in allowed)}")
        if isinstance(value, Step):
            earlier = shapes.get(value.case.test)
            if earlier and earlier.returns is None:
                faults.append(f"takes {key} from the product of the case at line {value.case.line}, and {earlier.name} returns nothing")
    for check in shape.checks:  # a check reads the product, or an input that the act can change (#965)
        read = {n for n in check.names if n in params or not hasattr(builtins, n)}  # `len(x)` reads x
        inputs = read & set(params)
        if inputs and not (read - inputs) and all(isinstance(case.inputs.get(n, params[n].default), IMMUTABLE) for n in inputs):
            faults.append(f"line {check.line} reads only {', '.join(sorted(inputs))}, a value the act cannot change; read what the act made")
    return faults


# ── ② IMPORT: each test's last asserts become checks, as pytest rewrites an assert ──


@dataclass
class Setup:
    """A `with` of the act, as the rewrite marked it: where it is, what it sets up, and the line of each statement inside."""

    file: str
    line: int
    text: str
    lines: list[int]


SETUPS: list[Setup] = []
MARK, LEFT, STEP = "__form_mark__", "__form_left__", "__form_step__"


def call_stmt(name: str, args: list[object], where: ast.stmt) -> ast.stmt:
    return ast.copy_location(ast.Expr(ast.Call(ast.Name(name, ast.Load()), [ast.Constant(a) for a in args], [])), where)


class Rewrite(ast.NodeTransformer):
    """`assert a == b, "m"` among a test's checks → `__form_check__("test", 0, lambda: (a, b), lambda: "m")`.

    Each setup's body gets a mark after each statement, and a call after the `with`: if the `with` let the test go on
    without its last statements, a raise inside it was swallowed (#979). A mark adds no frame to the `with`'s exit.
    """

    def __init__(self, file: str) -> None:
        self.file = file

    def visit_FunctionDef(self, node: ast.FunctionDef) -> ast.FunctionDef:
        _, asserts, _ = split(node)
        place = {id(s): k for k, s in enumerate(asserts)}

        def fix(body: list[ast.stmt], top: bool = False) -> list[ast.stmt]:
            out: list[ast.stmt] = []
            for stmt in body:
                if id(stmt) in place:
                    test = stmt.test
                    assert isinstance(stmt, ast.Assert)
                    pair = ast.Tuple([test.left, test.comparators[0]], ast.Load()) if isinstance(test, ast.Compare) and len(test.comparators) == 1 else test
                    message = stmt.msg if stmt.msg is not None else ast.Constant(None)
                    call = ast.Call(ast.Name(HOOK, ast.Load()), [ast.Constant(node.name), ast.Constant(place[id(stmt)]), lam(pair), lam(message)], [])
                    out.append(ast.copy_location(ast.Expr(call), stmt))
                elif is_setup(stmt):
                    assert isinstance(stmt, ast.With)
                    sid = len(SETUPS)
                    SETUPS.append(Setup(self.file, stmt.lineno, ", ".join(ast.unparse(i) for i in stmt.items), [s.lineno for s in stmt.body]))
                    inner = fix(stmt.body)
                    marked: list[ast.stmt] = [call_stmt(MARK, [sid, -1], stmt)]
                    for k, s in enumerate(inner):
                        marked += [s, call_stmt(MARK, [sid, k], s)]
                    stmt.body = marked
                    out += [stmt, call_stmt(LEFT, [sid], stmt)]
                else:
                    out.append(stmt)
                    if top and not isinstance(stmt, ast.Return):
                        out.append(call_stmt(STEP, [], stmt))
            return out

        node.body = fix(node.body, top=True)
        return node


def lam(body: ast.expr) -> ast.Lambda:
    return ast.Lambda(ast.arguments([], [], None, [], [], None, []), body)


def load(path: Path) -> types.ModuleType:
    """Import a test file, rewritten, under a module name of its own: two files named alike stay two modules."""
    real, name, n = os.path.realpath(path), path.stem, 1
    while name in sys.modules and os.path.realpath(getattr(sys.modules[name], "__file__", None) or "") != real:
        n += 1
        name = f"{path.stem}_{n}"
    if name in sys.modules:
        return sys.modules[name]
    tree = ast.parse(path.read_text(), str(path))
    tree.body = [Rewrite(str(path)).visit(s) if isinstance(s, ast.FunctionDef) and s.name.startswith("test") else s for s in tree.body]
    module = types.ModuleType(name)
    module.__file__ = str(path)
    vars(module)[HOOK] = hook
    vars(module)[MARK] = mark
    vars(module)[LEFT] = left
    vars(module)[STEP] = step
    sys.modules[name] = module
    try:
        exec(compile(ast.fix_missing_locations(tree), str(path), "exec"), vars(module))
    except BaseException:
        del sys.modules[name]
        raise
    return module


# ── ③ RUN: one forked process for each case; each later check read alone in a fork, then all in order ──


class Blocked(BaseException):
    pass


@dataclass
class Run:
    """What one case's process holds: the products of replayed steps, each stock it drew, and its checks so far."""

    shapes: dict[Any, Shape]
    names: dict[int, str]
    fd: int = -1
    memo: dict[int, object] = field(default_factory=dict)
    stocks: dict[type, tuple[Stock[Any], object, object]] = field(default_factory=dict)  # stock type → (stock, value, checkpoint)
    stock_inputs: dict[str, Stock[Any]] = field(default_factory=dict)
    replaying: list[str] = field(default_factory=list)  # the label of each step being replayed, innermost last
    done: set[int] = field(default_factory=set)  # the checks of this case that ran
    reached: dict[int, int] = field(default_factory=dict)  # each open setup → the last statement inside it that ended
    guard: Guard | None = None
    only: int | None = None  # in a fork that reads one check alone: that check (#980)
    alone_fd: int = -1
    forked: bool = False  # the forks that read each later check alone are done (#980)
    alone: dict[int, dict[str, Any]] = field(default_factory=dict)  # each later check → what it read alone
    flagged: bool = False  # the act already failed for its setup or a hook: its paths are not judged (#980)
    case: Case[Any] | None = None  # the case this process runs (#1010)
    # #1010: each stage → its later stages in the fork tree
    tree: dict[int, list[Case[Any]]] = field(default_factory=dict)
    # this case's later stages, while it may still fork for them
    kids: list[Case[Any]] = field(default_factory=list)
    split: bool = False  # the stage forked for its own checks, or fell back (#1010)
    # this process is the stage's continuation: it reads the checks as a replay does
    cont: bool = False
    # forked from an earlier stage's continuation: its file's `before_each` already ran
    inherited: bool = False
    swallowed: str | None = None  # a setup of the act swallowed a raise: a replay would block on it
    # each open descriptor when the case started → what it points to
    fds: dict[str, str] = field(default_factory=dict)


# ── a setup: what it swallowed, and what its undo raised (#979) ─────────────────────


def mark(sid: int, k: int) -> None:
    if RUN:
        RUN[-1].reached[sid] = k
        verify()


def step() -> None:
    """After each statement of the act at the test's own level: is every hook the runner's own again?"""
    if RUN:
        verify()


def verify() -> None:
    guard = RUN[-1].guard if RUN else None
    if guard is not None:
        guard.verify()


def left(sid: int) -> None:
    """The test went on after a setup. If a statement inside it raised, the setup swallowed that raise."""
    if not RUN:
        return
    run, setup = RUN[-1], SETUPS[sid]
    verify()
    k = run.reached.pop(sid, None)
    if k is None or k >= len(setup.lines) - 2:  # all ran, or the last statement raised: nothing was skipped
        return
    raised, skipped = setup.lines[k + 1], setup.lines[k + 2 :]
    why = f"`with {setup.text}` swallowed a raise at line {raised}, so line{'s' * (len(skipped) > 1)} {', '.join(map(str, skipped))} never ran"
    if run.replaying:
        raise Blocked(f"{run.replaying[-1]}: {why}")
    run.flagged = True
    run.swallowed = why
    emit(run.fd, check="setup", line=0, status="fail", why=why, trace=[f"at {rel(setup.file)}:{setup.line}"])


def setup_at(error: BaseException, file: str, test: str, run: Run) -> tuple[str, str] | None:
    """When the error came from a setup's own `with` line, not from a statement inside it: its entry, or its undo."""
    shown = [f for f in traceback.extract_tb(error.__traceback__) if os.path.realpath(f.filename) == os.path.realpath(file) and f.name == test]
    if not shown:
        return None
    line = shown[-1].lineno
    for sid, setup in enumerate(SETUPS):
        if os.path.realpath(setup.file) == os.path.realpath(file) and setup.line <= (line or 0) < (setup.lines[0] if setup.lines else setup.line + 1):
            reached = run.reached.get(sid)
            if reached is None:
                return "setup", f"setting up `with {setup.text}` at line {setup.line} raised {type(error).__name__}: {error}"
            if reached == len(setup.lines) - 1:
                return "undo", f"the undo of `with {setup.text}` at line {setup.line} raised {type(error).__name__}: {error}"
    return None


# ── the guard: the act may not replace a hook that the runner sees a fault through (#979) ──

GUARDED: dict[str, tuple[str, ...]] = {"threading": ("excepthook",), "sys": ("unraisablehook",), "warnings": ("filters", "showwarning", "_showwarnmsg")}
SEES = {
    "threading.excepthook": "an error that no code sees in a thread",
    "sys.unraisablehook": "an error that no code can catch, such as one in `__del__`",
    "warnings": "a warning that the test does not expect",
    "signal": "a case that runs too long",
}
MINE = {os.path.realpath(__file__), os.path.realpath(HERE / "cases.py")}
STDLIB = os.path.realpath(sysconfig.get_paths()["stdlib"])


GUARD_MODE = os.environ.get("FORM_GUARD", "line")  # line (agreed): fail when a line of the test runs under it · swap: fail at the swap


class Guard:
    """`swap` fails the case the moment the act replaces a hook. `line` only notes where, and fails the case when a
    hook is still not the runner's own after a statement of the test: a window that a library opens and closes
    inside one call of the test passes (freezegun's `catch_warnings` inside `freeze_time`)."""

    def __init__(self, run: Run) -> None:
        self.run = run
        self.on = False
        self.tripped: set[str] = set()
        self.pending: dict[str, tuple[str, str]] = {}  # family → (name, where), at the swap (line mode)
        self.filters: list[Any] = []
        self.own: dict[str, object] = {}

    def start(self) -> None:
        self.filters = list(warnings.filters)
        self.own = {"threading": threading.excepthook, "sys": sys.unraisablehook, "warnings": warnings.showwarning, "signal": signal.getsignal(signal.SIGALRM)}
        self.on = True

    def trip(self, name: str) -> None:
        family = name.split(".")[0]
        if not self.on or OWNED[0] or self.run.replaying or family in self.tripped:
            return
        where = next((f for f in reversed(traceback.extract_stack()[:-1]) if os.path.realpath(f.filename) not in MINE and not os.path.realpath(f.filename).startswith(STDLIB + os.sep)), None)
        at = f"at {rel(where.filename)}:{where.lineno}  {where.line}" if where else ""
        if GUARD_MODE == "line":
            self.pending.setdefault(family, (name, at))
            return
        self.fail(family, name, at)

    def fail(self, family: str, name: str, at: str) -> None:
        self.tripped.add(family)
        self.run.flagged = True
        sees = SEES.get(name) or SEES[family]
        emit(self.run.fd, check="hook", line=0, status="fail", why=f"the act replaced {name}; the runner sees {sees} through it, so while it is replaced the case cannot show one", trace=[at] if at else [])

    def changed(self) -> list[str]:
        """The families whose hook is not the runner's own right now."""
        out = []
        if threading.excepthook is not self.own["threading"]:
            out.append("threading")
        if sys.unraisablehook is not self.own["sys"]:
            out.append("sys")
        if list(warnings.filters) != self.filters or warnings.showwarning is not self.own["warnings"]:
            out.append("warnings")
        if signal.getsignal(signal.SIGALRM) != self.own["signal"] or "signal" in self.pending:
            out.append("signal")  # a cancelled alarm cannot be read back, so a call to it counts
        return out

    def verify(self) -> None:
        if not self.on or OWNED[0] or self.run.replaying:
            return
        if GUARD_MODE != "line":
            if list(warnings.filters) != self.filters:
                self.trip("warnings.filters")
            return
        for family in self.changed():
            if family not in self.tripped:
                name, at = self.pending.get(family, (family, ""))
                self.fail(family, name, at)

    def install(self) -> None:
        for name, attrs in GUARDED.items():
            module = sys.modules[name]
            guard = self

            class Guarded(types.ModuleType):
                def __setattr__(self, key: str, value: object, _attrs: tuple[str, ...] = attrs, _name: str = name) -> None:
                    if key in _attrs:
                        guard.trip(f"{_name}.{key}")
                    types.ModuleType.__setattr__(self, key, value)

            module.__class__ = Guarded
        real_alarm, real_signal, real_timer = signal.alarm, signal.signal, signal.setitimer

        def alarm(seconds: int) -> int:
            self.trip("signal.alarm")
            return real_alarm(seconds)

        def on_signal(signum: int, handler: Any) -> Any:
            if signum == signal.SIGALRM:
                self.trip("signal.signal(SIGALRM, …)")
            return real_signal(signum, handler)

        def timer(which: int, seconds: float, interval: float = 0.0) -> tuple[float, float]:
            if which == signal.ITIMER_REAL:
                self.trip("signal.setitimer(ITIMER_REAL, …)")
            return real_timer(which, seconds, interval)

        signal.alarm, signal.signal, signal.setitimer = alarm, on_signal, timer  # type: ignore[assignment]
        for fn in ("simplefilter", "filterwarnings", "resetwarnings"):  # they change the filters in place: trip where they are called
            real = getattr(warnings, fn)

            def changed(*args: Any, _real: Any = real, _fn: str = fn, **kwargs: Any) -> Any:
                self.trip(f"warnings.filters (by warnings.{_fn})")
                return _real(*args, **kwargs)

            setattr(warnings, fn, changed)


# ── the paths of the act, seen while the case runs (#980) ───────────────────────────

MON = sys.monitoring
TOOL = 3  # no tool of Python's own: a debugger or coverage of the code under test keeps its id
BODY_ONCE = "the act has one path, so each line of it runs in every case"
BOTH_WAYS = "the act has one path, so each `if`, `x if c else y`, `and`, `or`, `while` and `except` in it takes both ways in a case, or is not there"


class Paths:
    """Which lines and branches of the test's own code run, in the case's process. Code is the test's own when its file
    sits in the test file's folder. It is part of the act when the test calls it, or something the test calls calls it:
    the test, a helper of the file or of the folder, a class it builds, a reading of a check. Code of the test's own that
    the code under test calls, such as a callback, is a value: its paths are free, but an assert in it is not.

    Each location is switched off once it has told what it can: a line at its first run, a branch once it went both
    ways, a loop once it ran a round. So a loop of a million rounds costs about what it costs with no watch."""

    def __init__(self, run: Run, shape: Shape) -> None:
        self.run, self.shape = run, shape
        self.root = os.path.realpath(os.path.dirname(shape.file)) + os.sep
        self.ours: dict[types.CodeType, bool] = {}
        self.act: set[types.CodeType] = set()
        self.lines: dict[types.CodeType, set[int]] = {}
        self.ways: dict[tuple[types.CodeType, int], set[int]] = {}
        self.loops: set[tuple[types.CodeType, int]] = set()  # a loop that was reached
        self.entered: set[tuple[types.CodeType, int]] = set()  # a loop that ran a round
        self.kinds: dict[types.CodeType, dict[int, tuple[str, int]]] = {}
        self.asserts: dict[str, set[int]] = {}
        self.faults: list[str] = []
        self.checks = {c.line for c in shape.checks}
        self.test = shape.fn.__code__
        self.after_raise = self.lines_after_raise()
        self.on = False

    def lines_after_raise(self) -> set[int]:
        """The lines of the act that start a statement after a `with raises(…):`: none of them may run once it caught."""
        if not self.shape.raises_spans:
            return set()
        end = max(e for _, e in self.shape.raises_spans)
        act, _, _ = split(self.shape.node)
        out: set[int] = set()

        def walk(stmts: list[ast.stmt]) -> None:
            for s in stmts:
                if isinstance(s, ast.With):
                    walk(s.body)
                elif s.lineno > end and s.lineno not in self.checks and not isinstance(s, ast.Return):
                    out.add(s.lineno)

        walk(act)
        return out

    def is_ours(self, code: types.CodeType) -> bool:
        if (known := self.ours.get(code)) is None:
            path = os.path.realpath(code.co_filename)
            known = self.ours[code] = path.startswith(self.root) and path not in MINE and code.co_name != "<module>"
        return known

    def assert_lines(self, file: str) -> set[int]:
        if file not in self.asserts:
            try:
                tree = ast.parse(Path(file).read_text(), file)
                self.asserts[file] = {n.lineno for n in ast.walk(tree) if isinstance(n, ast.Assert)}
            except (OSError, SyntaxError):
                self.asserts[file] = set()
            if os.path.realpath(file) == os.path.realpath(self.shape.file):
                self.asserts[file] -= self.checks
        return self.asserts[file]

    def kind_of(self, code: types.CodeType) -> dict[int, tuple[str, int]]:
        """Each branch of the code: a loop's head (with where its body starts), the test of an `except`, the exit of a `with`.
        On 3.12 an exhausted `FOR_ITER` lands past its `END_FOR`, not on its argument, so a round is told by the body."""
        if code not in self.kinds:
            found: dict[int, tuple[str, int]] = {}
            prev = None
            listed = list(dis.get_instructions(code))
            for ins, nxt in zip(listed, [*listed[1:], None]):
                if ins.opname == "FOR_ITER" and nxt is not None:
                    found[ins.offset] = ("loop", nxt.offset)
                elif ins.opname.startswith("POP_JUMP_IF") and prev is not None:
                    found[ins.offset] = ("except", 0) if prev.opname == "CHECK_EXC_MATCH" else ("with", 0) if prev.opname == "WITH_EXCEPT_START" else ("if", 0)
                prev = ins
            self.kinds[code] = found
        return self.kinds[code]

    def where(self, code: types.CodeType, line: int) -> str:
        return f"{rel(code.co_filename)}:{line}" + (f" ({code.co_name})" if code is not self.test else "")

    def py_start(self, code: types.CodeType, _offset: int) -> object:
        if not self.on:
            return MON.DISABLE
        if not self.is_ours(code):
            return MON.DISABLE
        if self.run.replaying:
            return None
        frame = sys._getframe(1)  # walk past the dispatch to the frame that starts, then to its caller
        while frame is not None and frame.f_code is not code:
            frame = frame.f_back
        frame = frame.f_back if frame is not None else None
        caller = frame.f_code if frame is not None else None
        if code is self.test or caller in self.act:
            self.act.add(code)
        elif caller is not None and os.path.realpath(caller.co_filename) == os.path.realpath(__file__):
            if code.co_name == "<lambda>" and code.co_firstlineno in self.checks and code.co_filename == self.test.co_filename:
                self.act.add(code)  # a check's reading, which the runner calls
        return None

    def line(self, code: types.CodeType, line: int) -> object:
        if not self.on:
            return MON.DISABLE
        if not self.is_ours(code):
            return MON.DISABLE
        if self.run.replaying:
            return None
        self.lines.setdefault(code, set()).add(line)
        if line in self.assert_lines(code.co_filename):
            self.faults.append(f"an assert ran in the act at {self.where(code, line)}: the act ends at the first check, and a function handed to the code may swallow an assert; keep what it sees, and check that at the end")
        if code is self.test and RAISED[0] and line in self.after_raise:
            self.faults.append(f"line {line} ran after the expected raise: `with raises(…):` ends the act, since after a raise the act has nothing left to run")
        return MON.DISABLE

    def branch(self, code: types.CodeType, src: int, dst: int) -> object:
        if not self.on:
            return MON.DISABLE
        if not self.is_ours(code):
            return MON.DISABLE
        if self.run.replaying:
            return None
        if code not in self.act:
            return MON.DISABLE  # a callback's paths are free
        what, body_at = self.kind_of(code).get(src, ("if", 0))
        if what == "with":
            return MON.DISABLE
        if what == "loop":
            if dst == body_at:
                self.entered.add((code, src))
                return MON.DISABLE
            self.loops.add((code, src))
            return None
        ways = self.ways.setdefault((code, src), set())
        ways.add(dst)
        return MON.DISABLE if len(ways) == 2 else None

    def start(self) -> None:
        WATCH[:] = [self]
        self.on = True

    def stop(self) -> None:
        self.on = False

    def judge(self) -> list[str]:
        out = list(dict.fromkeys(self.faults))
        line_of = lambda code, src: next((ln for s, e, ln in code.co_lines() if s <= src < e and ln), code.co_firstlineno)  # noqa: E731
        for code in sorted(self.act, key=lambda c: (c.co_filename, c.co_firstlineno)):
            body = {ln for _, _, ln in code.co_lines() if ln is not None and ln != code.co_firstlineno}
            missed = sorted(body - self.lines.get(code, set()))
            if missed:
                out.append(f"line{'s' * (len(missed) > 1)} {', '.join(map(str, missed))} of {rel(code.co_filename)}{'' if code is self.test else f' ({code.co_name})'} never ran: {BODY_ONCE}")
            for (c, src), ways in self.ways.items():
                if c is code and len(ways) == 1 and line_of(code, src) not in self.assert_lines(code.co_filename):  # an assert is told once, above
                    what = self.kind_of(code).get(src, ("if", 0))[0]
                    label = "the `except`" if what == "except" else "the branch"
                    out.append(f"{label} at {self.where(code, line_of(code, src))} went one way: {BOTH_WAYS}")
            for c, src in sorted(self.loops - self.entered, key=lambda x: x[1]):
                if c is code:
                    out.append(f"the loop at {self.where(code, line_of(code, src))} ran no round: {BODY_ONCE}")
        return out


WATCH: list[Paths] = []  # the watch of this case, set in the case's process
ARMED: set[int] = set()


def arm(root: str) -> None:
    """Set the watch's events on the code of one test folder, once, in the runner's own process, before any case
    forks. Each change of events makes every code object of a process instrument itself again at its next run, which
    cost about 10 ms a case on loguru's tests when each case's process set them. A case's process sets nothing: it
    inherits the events, and only points them at its own watch."""
    if not ARMED:
        MON.use_tool_id(TOOL, "purba-980")
        on = lambda name: lambda *a: getattr(WATCH[-1], name)(*a) if WATCH else None  # noqa: E731  (no watch: the runner's own process)
        MON.register_callback(TOOL, MON.events.PY_START, on("py_start"))
        MON.register_callback(TOOL, MON.events.LINE, on("line"))
        MON.register_callback(TOOL, MON.events.BRANCH, on("branch"))
    found: dict[int, types.CodeType] = {}

    def add(code: types.CodeType) -> None:
        path = os.path.realpath(code.co_filename)
        if id(code) not in found and path.startswith(root) and path not in MINE:
            found[id(code)] = code
            for const in code.co_consts:
                if isinstance(const, types.CodeType):
                    add(const)

    def scan(value: object, depth: int = 0) -> None:
        fn = getattr(getattr(value, "__func__", value), "__wrapped__", getattr(value, "__func__", value))
        if isinstance(fn, types.FunctionType):
            add(fn.__code__)
        elif isinstance(value, type) and depth < 3:
            for member in list(vars(value).values()):
                scan(member, depth + 1)
        elif isinstance(value, property):
            for part in (value.fget, value.fset, value.fdel):
                scan(part, depth + 1)

    for module in list(sys.modules.values()):
        path = getattr(module, "__file__", None)
        if path and os.path.realpath(path).startswith(root) and os.path.realpath(path) not in MINE:
            for value in list(vars(module).values()):
                scan(value)
    wanted = MON.events.PY_START | MON.events.LINE | MON.events.BRANCH
    for code in found.values():
        if id(code) not in ARMED:
            MON.set_local_events(TOOL, code, wanted)
            ARMED.add(id(code))


RUN: list[Run] = []  # the run of this process, set in the case's process


def short(value: object) -> str:
    if isinstance(value, types.FunctionType):
        return f"<lambda at line {value.__code__.co_firstlineno}>" if value.__name__ == "<lambda>" else value.__name__
    text = repr(value)
    return text if len(text) <= LONG else text[: LONG // 2] + " … " + text[-LONG // 2 :]  # the end survives: a traceback ends with its point


def kind(value: object) -> str:
    t = type(value)
    if t is bool:
        return "bool"
    base = next((b for b in (str, int, float, bytes, list, tuple, dict, set) if issubclass(t, b) and t is not b), None)
    return f"{t.__name__}, a subclass of {base.__name__}" if base else t.__name__


def a(text: str) -> str:
    return f"{'an' if text[0] in 'aeiou' else 'a'} {text}"


def differs(expected: object, reading: object, at: str = "") -> str | None:
    """None when the reading equals the expected value; else why not ("" when only the values differ)."""
    if type(expected) is bool or type(reading) is bool:
        if type(expected) is not type(reading):
            return f"{at}{a(kind(reading))} is not {a(kind(expected))}"
        return None if expected == reading else ""
    if isinstance(expected, float) and isinstance(reading, float) and math.isnan(expected) and math.isnan(reading):
        return None
    if not isinstance(reading, type(expected)):
        return f"{at}{a(kind(reading))} is not {a(kind(expected))}"
    if isinstance(expected, (list, tuple)):
        assert isinstance(reading, (list, tuple))
        if len(reading) != len(expected):
            return f"{at}{len(reading)} items, not {len(expected)}"
        return next((d for i, (e, r) in enumerate(zip(expected, reading)) if (d := differs(e, r, f"{at}[{i}]: ")) is not None), None)
    if isinstance(expected, dict):
        assert isinstance(reading, dict)
        if set(reading) != set(expected):
            return f"{at}keys {sorted(map(repr, reading))}, not {sorted(map(repr, expected))}"
        return next((d for k in expected if (d := differs(expected[k], reading[k], f"{at}[{k!r}]: ")) is not None), None)
    if reading == expected:
        return None
    if isinstance(expected, str) and isinstance(reading, str):
        at_ = next((i for i, (x, y) in enumerate(zip(expected, reading)) if x != y), min(len(expected), len(reading)))
        return f"{at}they differ from character {at_}: expected {expected[at_:at_ + 20]!r}, read {reading[at_:at_ + 20]!r}"
    return ""


def verdict(check: Check, left: object, right: object) -> str | None:
    """None when the check holds; else why not. `==` and `!=` compare by type, then by value."""
    reading, expected = (left, right) if check.reading_left else (right, left)
    if check.op is ast.Eq:
        return differs(expected, reading)
    if check.op is ast.NotEq:
        return None if differs(expected, reading) is not None else "they are equal"
    symbol, holds = OPS[check.op]
    return None if holds(left, right) else f"`{short(left)} {symbol} {short(right)}` does not hold"


def emit(fd: int, **result: object) -> None:
    if fd >= 0:  # a fork that reads one check alone says nothing but that check's verdict (#980)
        os.write(fd, (json.dumps(result) + "\n").encode())


def frames(error: BaseException, file: str) -> list[str]:
    shown = [f for f in traceback.extract_tb(error.__traceback__) if os.path.realpath(f.filename) == os.path.realpath(file)]
    return [f"at {rel(f.filename)}:{f.lineno}  {f.line}" for f in shown[-2:]]


def check_one(shape: Shape, check: Check, pair: Callable[[], Any], message: Callable[[], Any]) -> dict[str, Any]:
    """Read one value and compare it, in this process: only the verdict, and the two texts of a failure, cross back."""
    try:
        left, right = pair()
        asks = message()
    except Exception as error:
        return dict(check=check.label, line=check.line, status="fail", why=f"reading it raised {type(error).__name__}: {error}", trace=frames(error, shape.file))
    why = verdict(check, left, right)
    reading, expected = (left, right) if check.reading_left else (right, left)
    if why is None:
        return dict(check=check.label, line=check.line, status="pass", reading=short(reading))
    if not why and short(expected) == short(reading):
        why = f"same text, types {kind(expected)} and {kind(reading)}"
    return dict(check=check.label, line=check.line, status="fail", asks=asks, expected=f"{short(expected)} ({kind(expected)})", reading=f"{short(reading)} ({kind(reading)})", why=why)


def read_guarded(run: Run, shape: Shape, check: Check, pair: Callable[[], Any], message: Callable[[], Any]) -> dict[str, Any]:
    """One check, with each stock of the case watched: a fingerprint before and after it, or else a checkpoint that is
    restored after it. A check whose reading changed a stock is refused, by the stock, not by the name it reads (#980)."""
    prints, marks = [], []
    for stock_type, (stock, value, _) in run.stocks.items():
        if isinstance(stock, Fingerprints):
            prints.append((stock_type, stock, value, stock.fingerprint(value)))
        elif isinstance(stock, Checkpoints):
            marks.append((stock, value, stock.checkpoint(value)))
    result = check_one(shape, check, pair, message)
    for stock, value, point in marks:
        stock.restore(value, point)
    for stock_type, stock, value, before in prints:
        if stock.fingerprint(value) != before:
            return dict(check=check.label, line=check.line, status="refused", why=f"reading it changed the stock {stock_type.__name__}: a check reads what the act made, and changes nothing")
    return result


def fork_alone(run: Run, shape: Shape, first: int) -> bool:
    """Before the first check reads, fork once for each later check, one at a time. Each fork runs the rest of the test
    and reads only its own check, so it reads that check alone. True in a fork, which then goes on with the test (#980)."""
    run.forked = True
    for j in range(first + 1, len(shape.checks)):
        flush()
        r, w = os.pipe()
        pid = os.fork()
        if pid == 0:
            os.close(r)
            run.only, run.alone_fd, run.fd = j, w, -1
            if run.guard:
                run.guard.on = False
            return True
        os.close(w)
        data = b""
        while chunk := os.read(r, 65536):
            data += chunk
        os.close(r)
        os.waitpid(pid, 0)
        if data.strip():
            run.alone[j] = json.loads(data.decode().splitlines()[-1])
    return False


def is_handle(value: object) -> bool:
    if isinstance(value, mmap.mmap):
        return True
    try:
        return isinstance(getattr(value, "fileno")(), int)
    except Exception:
        return False


def hook(test: str, k: int, pair: Callable[[], Any], message: Callable[[], Any]) -> None:
    """A check, where the test wrote its assert. On a replay it only compares; in the case, every check runs."""
    run = RUN[-1]
    caller = sys._getframe(1)
    shape = next(s for s in run.shapes.values() if s.name == test and os.path.realpath(s.file) == os.path.realpath(caller.f_code.co_filename))
    check = shape.checks[k]
    if run.kids and not run.split and not run.replaying and run.only is None:
        fork_stage(run)  # #1010: the act ended at this check; a continuation now replays
    if run.replaying:
        left, right = pair()
        if (why := verdict(check, left, right)) is not None:
            reading, expected = (left, right) if check.reading_left else (right, left)
            raise Blocked(f"{run.replaying[-1]} › {check.label} diverged on replay: expected {short(expected)}, read {short(reading)}" + (f" ({why})" if why else ""))
        return
    if run.only is not None:  # a fork that reads one check alone: the others it passes by (#980)
        if k == run.only:
            os.write(run.alone_fd, (json.dumps(read_guarded(run, shape, check, pair, message)) + "\n").encode())
            os._exit(0)
        return
    if not run.forked and os.environ.get("FORM_ALONE", "on") == "on" and fork_alone(run, shape, k):  # off only to measure
        return  # in the fork: go on to the check it reads
    run.done.add(k)
    if run.guard:
        run.guard.verify()
    local = caller.f_locals
    readers = lambda n: sum(n in c.names for c in shape.checks)  # noqa: E731
    held = sorted(n for n in check.names if n in local and is_handle(local[n]) and readers(n) >= 2)
    if held:
        emit(run.fd, check=check.label, line=check.line, status="refused", why=f"{held[0]} is an open file, socket, pipe or mmap, and more than one check reads it: a fork cannot seal it")
        return
    # every check reads in place, in order; each later one was also read alone, before any check read (#980)
    result = read_guarded(run, shape, check, pair, message)
    alone = run.alone.get(k)
    if alone and {alone["status"], result["status"]} == {"pass", "fail"}:
        before = ", ".join(str(c.line) for c in shape.checks[:k])
        result = dict(
            check=check.label, line=check.line, status="refused",
            why=f"it {'holds' if alone['status'] == 'pass' else 'fails'} when read alone, and {'holds' if result['status'] == 'pass' else 'fails'} after the checks before it (line{'s' * (k > 1)} {before}): a reading before it changed what it reads, and a check reads, it does not act",
        )
    emit(run.fd, **result)


# ── #1010: a chain as a fork tree; a refused earlier stage blocks each later one ──

CHAIN = ["replay"]  # or "tree", with `--tree`
FDS = [os.environ.get("FORM_FDS", "on")]  # off only to show what the fallback for an open fd guards
JOBS: list[str] = []  # the folder of each case's results and output, for this run
REAL_ALARM = signal.alarm  # before the guard wraps it: a fork's time limit is the runner's


def job(case: Case[Any], kind: str) -> str:
    return os.path.join(JOBS[0], f"{id(case)}.{kind}")


def results_of(case: Case[Any]) -> list[dict[str, Any]] | None:
    try:
        with open(job(case, "res")) as f:
            return [json.loads(line) for line in f.read().splitlines()]
    except FileNotFoundError:
        return None


def outcome(results: list[dict[str, Any]]) -> str:
    """A case's status, as `report` counts it."""
    status = "pass"
    for res in results:
        if res["status"] == "blocked":
            status = "blocked" if status == "pass" else status
        elif res["status"] != "pass":
            status = "refused" if res["status"] == "refused" or status == "refused" else "fail"
    return status


def label_of(case: Case[Any], run: Run) -> str:
    return run.names.get(id(case), f"the case at line {case.line}")


def refused_upstream(case: Case[Any], run: Run) -> str | None:
    """The decided rule: a refused earlier stage blocks each later stage, and the block names
    it. Each earlier stage ran before this one, in both modes, so its results are in its file."""
    seen: set[int] = set()

    def walk(c: Case[Any]) -> str | None:
        for v in c.inputs.values():
            if isinstance(v, Step) and id(v.case) not in seen:
                seen.add(id(v.case))
                found = results_of(v.case)
                if found is not None and outcome(found) == "refused":
                    return f"{label_of(v.case, run)} was refused"
                if (why := walk(v.case)) is not None:
                    return why
        return None

    return walk(case)


def block_case(fd: int, shape: Shape, why: str) -> None:
    for c in shape.checks:
        emit(fd, check=c.label, line=c.line, status="blocked", why=why)
    if not shape.checks:  # a stage with no check of its own is still blocked, not passed
        emit(fd, check="case", line=0, status="blocked", why=why)


def open_fds() -> dict[str, str]:
    found: dict[str, str] = {}
    for fd in os.listdir("/proc/self/fd"):
        try:
            found[fd] = os.readlink(f"/proc/self/fd/{fd}")
        except OSError:
            pass  # the descriptor of the listing itself, closed by now
    return found


def fallback_reason(run: Run) -> str | None:
    if run.stocks:
        stocks = ", ".join(sorted(t.__name__ for t in run.stocks))
        return f"the chain reads a stock ({stocks}), whose state a fork does not copy"
    alive = [t.name for t in threading.enumerate() if t is not threading.main_thread()]
    if alive:
        return (
            f"a thread other than the main one is alive after the act ({alive[0]}), "
            "so a fork is unsafe (#978)"
        )
    if FDS[0] == "on":
        opened = sorted(
            (fd, to) for fd, to in open_fds().items()
            if run.fds.get(fd) != to and not to.startswith("/proc/")
        )
        if opened:  # a fork shares its offset between the later stages; replay does not
            fd, to = opened[0]
            return (
                f"the act left a file, socket or pipe open (fd {fd} → {to}), "
                "whose offset a fork shares"
            )
    return None


def fork_stage(run: Run) -> None:
    """The act of a stage with later stages has run once. Fork once for the stage's own checks,
    which then run as any case's do. This process waits for that fork, and goes on as the stage's
    continuation: it reads the checks as a replay does, takes the product, and forks once for each
    later stage (`spawn_later`). On a fallback the stage runs as any case, and the runner replays
    its later stages after it."""
    assert run.case is not None
    run.split = True
    if (why := fallback_reason(run)) is not None:
        Path(job(run.case, "fallback")).write_text(why)
        run.kids = []
        return
    flush()
    pid = os.fork()
    if pid == 0:  # the stage's own checks
        REAL_ALARM(SECONDS)
        run.kids = []
        return
    wait_case(pid, run.case)  # its outcome is in its file before any later stage reads it
    run.cont = True
    run.fd = -1
    if run.guard:
        run.guard.on = False
    run.replaying.append(label_of(run.case, run))


def wait_case(pid: int, case: Case[Any]) -> None:
    _, status = os.waitpid(pid, 0)
    if os.WIFSIGNALED(status):
        fd = os.open(job(case, "res"), os.O_WRONLY | os.O_CREAT | os.O_APPEND)
        why = f"the case's process died: signal {os.WTERMSIG(status)} (a case gets {SECONDS} s)"
        emit(fd, check="act", line=0, status="fail", why=why)
        os.close(fd)


def block_subtree(case: Case[Any], run: Run, why: str) -> None:
    fd = os.open(job(case, "res"), os.O_WRONLY | os.O_CREAT | os.O_APPEND)
    block_case(fd, run.shapes[case.test], why)
    os.close(fd)
    for kid in run.tree.get(id(case), []):
        block_subtree(kid, run, why)


def spawn_later(run: Run, stage: Case[Any], why: str | None) -> None:
    """In the stage's continuation: fork once for each later stage, which takes the product
    from this process."""
    REAL_ALARM(0)
    for kid in run.tree.get(id(stage), []):
        if why is not None:
            block_subtree(kid, run, why)
            continue
        flush()
        pid = os.fork()
        if pid == 0:
            child = Run(run.shapes, run.names, memo=dict(run.memo), tree=run.tree, inherited=True)
            child.kids = list(run.tree.get(id(kid), []))
            case_process(kid, child)
        wait_case(pid, kid)


def resolve(name: str, given: object, run: Run, top: bool) -> object:
    if isinstance(given, Step):
        return replay(given.case, run)
    if not isinstance(given, Use):
        return given
    factory = given.factory
    if isinstance(factory, type) and issubclass(factory, Stock):
        if factory not in run.stocks:  # one sample or checkpoint of each stock, for the whole case and its replays
            stock = factory()
            if isinstance(stock, Samples):
                run.stocks[factory] = (stock, stock.sample(), None)
            else:
                assert isinstance(stock, Checkpoints)
                value = stock.open()
                run.stocks[factory] = (stock, value, stock.checkpoint(value))
        stock, value, _ = run.stocks[factory]
        if top:
            run.stock_inputs[name] = stock
        return value
    return factory()


def build(case: Case[Any], run: Run, top: bool) -> dict[str, object]:
    built: dict[str, object] = {}
    for name, p in inspect.signature(case.test).parameters.items():
        built[name] = resolve(name, case.inputs[name] if name in case.inputs else p.default, run, top)
    return built


def replay(step: Case[Any], run: Run) -> object:
    """Run an earlier case again, once for each case, with its checks. A step that reads differently blocks the case."""
    if id(step) in run.memo:
        return run.memo[id(step)]
    label = run.names.get(id(step), f"the case at line {step.line}")
    inputs = build(step, run, top=False)
    run.replaying.append(label)
    try:
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")
            product = step.test(**inputs)
    except (Blocked, Missed) as why:
        raise Blocked(str(why)) from None
    except Exception as error:
        raise Blocked(f"the step {label} raised {type(error).__name__}: {error}") from None
    finally:
        run.replaying.pop()
    run.memo[id(step)] = product
    return product


def in_case(case: Case[Any], run: Run) -> None:
    signal.alarm(SECONDS)
    RUN.append(run)
    run.case = case
    run.fds = open_fds()
    shape = run.shapes[case.test]
    lost: list[str] = []
    sys.unraisablehook = lambda u: lost.append(f"{type(u.exc_value).__name__}: {u.exc_value}")
    threading.excepthook = lambda h: lost.append(f"in a thread, {h.exc_type.__name__}: {h.exc_value}")

    def block_the_rest(why: str) -> None:
        for k, c in enumerate(shape.checks):
            if k not in run.done:
                emit(run.fd, check=c.label, line=c.line, status="blocked", why=why)

    try:
        for file, fn in [] if run.inherited else BEFORE:  # #1010: a forked later stage ran it
            if os.path.realpath(file) == os.path.realpath(shape.file):
                try:
                    fn()
                except Exception as error:
                    emit(run.fd, check=f"before_each({name_of(fn)})", line=0, status="fail", why=f"it raised {type(error).__name__}: {error}")
                    return block_the_rest("the case did not start")
        if (refused := refused_upstream(case, run)) is not None:
            return block_case(run.fd, shape, refused)
        try:
            inputs = build(case, run, top=True)
        except Blocked as why:
            return block_case(run.fd, shape, str(why))
        except Exception as error:
            emit(run.fd, check="inputs", line=0, status="fail", why=f"building an input raised {type(error).__name__}: {error}", trace=frames(error, shape.file))
            return block_the_rest("an input was not built")
        for name, value in inputs.items():  # a stand-in matches the type that the test declares
            allowed = annotation_types(shape.params[name].annotation, vars(sys.modules[case.test.__module__]))
            if allowed and isinstance(shape.params[name].default, Use) and name not in case.inputs and not fits(value, allowed):
                emit(run.fd, check="inputs", line=0, status="fail", why=f"the fixture of {name} gives {a(kind(value))}, and the test declares {' | '.join(t.__name__ for t in allowed)}")
                return block_the_rest("an input was not built")
        SEEN.clear()
        RAISED[0] = False
        error: BaseException | None = None
        guard = run.guard = Guard(run)
        guard.install()
        paths = Paths(run, shape)
        with warnings.catch_warnings(record=True) as seen:
            warnings.simplefilter("always")
            guard.start()
            paths.start()
            product: object = None
            stopped: str | None = None  # in a continuation: why its later stages are blocked
            try:
                product = case.test(**inputs)
            except (Missed, Exception) as raised:
                error = raised
            except Blocked as why:  # only a continuation reads its checks as a replay does
                if not run.cont:
                    raise
                stopped = str(why)
            finally:
                paths.stop()
                if run.only is not None:  # a fork that read one check alone, or never reached it: it says nothing more
                    os._exit(0)
                guard.verify()
                guard.on = False
        if run.kids and not run.split:  # #1010: the stage reached no check, so it forks at its end
            fork_stage(run)
        if run.cont:  # block the later stages as a replay would, or fork once for each
            label = run.replaying[-1]
            if stopped is None and run.swallowed is not None:
                stopped = f"{label}: {run.swallowed}"
            if stopped is None and error is not None:
                raised = f"the step {label} raised {type(error).__name__}: {error}"
                stopped = str(error) if isinstance(error, Missed) else raised
            run.memo[id(case)] = product
            spawn_later(run, case, stopped)
            flush()
            os._exit(0)
        if error is None and not run.flagged:  # #980: the paths of the act, and whether any check ran
            for why in paths.judge():
                emit(run.fd, check="path", line=0, status="refused", why=why)
            if not run.done and not shape.expects:
                emit(run.fd, check="checks", line=0, status="refused", why="no check ran: end the test with `assert reading == expected`, or expect a raise with `with raises(…):`")
        for text in SEEN:
            emit(run.fd, check=text.split(" ", 1)[0], line=0, status="pass", reading=text.split(" ", 1)[1])
        if error is not None:
            missed = isinstance(error, Missed)
            at = None if missed else setup_at(error, shape.file, shape.name, run)
            label, why = (error.form, str(error)) if isinstance(error, Missed) else at or ("act", f"the act raised {type(error).__name__}: {error}")
            emit(run.fd, check=label, line=0, status="fail", why=why, trace=frames(error, shape.file))
            block_the_rest("the act did not end as the test says")
        if error is None:
            block_the_rest("it never ran")
        for w in seen:
            emit(run.fd, check="warning", line=0, status="fail", why=f"a warning that the test does not expect: {w.category.__name__}: {w.message}", trace=[f"at {rel(w.filename)}:{w.lineno}"])
        for text in lost:
            emit(run.fd, check="lost error", line=0, status="fail", why=f"an error that no code saw: {text}")
    finally:
        for stock_type, (stock, value, point) in run.stocks.items():
            try:
                if isinstance(stock, Samples):
                    stock.discard(value)
                else:
                    assert isinstance(stock, Checkpoints)
                    stock.restore(value, point)
            except Exception as error:
                emit(run.fd, check="stock", line=0, status="fail", why=f"{stock_type.__name__} was not discarded or restored: {type(error).__name__}: {error}")


def case_process(case: Case[Any], run: Run) -> typing.NoReturn:
    """The body of a case's own process: its results and output go to files of its own."""
    fd = os.open(job(case, "res"), os.O_WRONLY | os.O_CREAT | os.O_APPEND)
    out = os.open(job(case, "out"), os.O_WRONLY | os.O_CREAT | os.O_APPEND)
    os.dup2(out, 1)
    os.dup2(out, 2)
    run.fd = fd
    if run.inherited:
        MON.restart_events()  # the earlier stage's act switched off the events its lines had raised
    try:
        in_case(case, run)
    except BaseException as error:  # a fault of the runner, shown as it is
        emit(fd, check="runner", line=0, status="fail", why=f"{type(error).__name__}: {error}")
    flush()
    os._exit(0)


def run_case(case: Case[Any], run: Run) -> None:
    flush()
    pid = os.fork()
    if pid == 0:
        case_process(case, run)
    wait_case(pid, case)


def read_case(case: Case[Any]) -> tuple[list[dict[str, Any]], str]:
    try:
        with open(job(case, "out"), "rb") as f:
            captured = f.read().decode(errors="replace")
    except FileNotFoundError:
        captured = ""
    return results_of(case) or [], captured


# ── ④ REPORT ────────────────────────────────────────────────────────────────────────


def rel(path: str | Path) -> str:
    try:
        return os.path.relpath(path)
    except ValueError:
        return str(path)


@dataclass
class Tally:
    passed: int = 0
    failed: int = 0
    refused: int = 0
    blocked: int = 0
    checks: int = 0
    files: int = 0


def show_inputs(case: Case[Any]) -> str:
    return " ".join(f"{k}={short(v)}" for k, v in case.inputs.items()) or "(none)"


def report(name: str, case: Case[Any] | None, results: list[dict[str, Any]], captured: str, tally: Tally, where: str) -> None:
    status = "pass"
    for res in sorted(results, key=lambda r: r["line"]):
        label = f"{name} › {res['check']}"
        if res["status"] == "pass":
            tally.checks += 1
            say(f"   ✓ {label}   {res.get('reading', '')}")
            continue
        if res["status"] == "blocked":
            status = "blocked" if status == "pass" else status
            say(f"   ⊘ {label}   blocked: {res['why']}")
            continue
        status = "refused" if res["status"] == "refused" or status == "refused" else "fail"  # #980: the test's fault outranks the code's
        mark = "!" if res["status"] == "refused" else "✗"
        line = f" · assert at line {res['line']}" if res["line"] else ""
        say(f"   {mark} {label}   {where}{line}")
        if res.get("asks"):
            say(f"       asks      {res['asks']}")
        for key in ("expected", "reading"):
            if key in res:
                say(f"       {key:<9} {res[key]}")
        if res.get("why"):
            say(f"       {'why' if 'expected' in res else 'what':<9} {res['why']}")
        for text in res.get("trace", []):
            say(f"                 {text}")
        if case is not None:
            say(f"       inputs    {show_inputs(case)}")
    if status != "pass" and captured.strip():
        say("       output    " + "\n                 ".join(captured.strip().splitlines()[-8:]))
    tally.passed += status == "pass"
    tally.failed += status == "fail"
    tally.refused += status == "refused"
    tally.blocked += status == "blocked"


def files_in(paths: list[str]) -> list[Path]:
    found: list[Path] = []
    for p in map(Path, paths or ["."]):
        if p.is_dir():
            found += sorted(f for f in p.rglob("test_*.py") if not any(part.startswith(".") or part == "__pycache__" for part in f.relative_to(p).parts))
        else:
            found.append(p)
    return found


def run_file(path: Path, tally: Tally) -> None:
    real = os.path.realpath(path)
    try:
        src = Source.read(path)
    except SyntaxError as error:
        say(f"── {rel(path)}\n   ! the file is refused: {error}")
        tally.files += 1
        return
    sys.path.insert(0, str(path.resolve().parent))
    before = len(CREATED)
    try:
        module = load(path)
        if os.environ.get("FORM_PATHS", "on") == "on":  # off only to measure what the watch costs
            arm(os.path.realpath(path.resolve().parent) + os.sep)
    except Exception as error:
        say(f"── {rel(path)}\n   ! the file is refused: importing it raised {type(error).__name__}: {error}\n     so none of its tests runs")
        tally.files += 1
        return
    scope = vars(module)
    tests = sorted(
        (v for k, v in scope.items() if k.startswith("test") and isinstance(v, types.FunctionType) and os.path.realpath(v.__code__.co_filename) == real),
        key=lambda f: f.__code__.co_firstlineno,
    )
    cases = [c for c in CREATED[before:] if os.path.realpath(c.file) == real] or [c for c in CREATED if os.path.realpath(c.file) == real]
    if not tests and not cases:
        return
    say(f"── {rel(path)}")
    for name, lines in src.bindings.items():  # two tests with one name: Python keeps the last
        if name.startswith("test") and len(lines) > 1 and isinstance(scope.get(name), types.FunctionType):
            say(f"   ! {name} is defined at lines {', '.join(map(str, lines))}; Python keeps only the last, so the test at line {lines[0]} would be lost")
            tally.refused += len(lines) - 1
    # #980: a case's name is read from the module after import, not from its source. `case()` keeps every case it
    # built; a case that no module name holds has none (bound again later, built by a loop, never bound).
    bound: dict[int, list[str]] = {}
    for k, v in scope.items():
        if isinstance(v, Case):
            bound.setdefault(id(v), []).append(k)
    names = {i: ns[0] for i, ns in bound.items() if len(ns) == 1}
    shapes = {t: shape_of(t, src) for t in tests}
    unnamed: dict[int, list[Case[Any]]] = {}
    for c in cases:
        if len(bound.get(id(c), [])) > 1:
            say(f"   ! a case of {name_of(c.test)} at line {c.line} has two names, {' and '.join(bound[id(c)])}: give each case one name")
            tally.refused += 1
        elif id(c) not in names:
            unnamed.setdefault(c.line, []).append(c)
        elif c.test not in shapes:
            say(f"   ! {names[id(c)]} at line {c.line} is a case of {name_of(c.test)}, which is no test of this file: a test is a function named `test…`")
            tally.refused += 1
    for line, lost in unnamed.items():
        count = f"{len(lost)} cases" if len(lost) > 1 else "a case"
        say(f"   ! {count} of {name_of(lost[0].test)} at line {line} {'have' if len(lost) > 1 else 'has'} no name, so the run cannot show {'them' if len(lost) > 1 else 'it'}: bind each case to a name of its own, `name = case(test, …)`, once")
        tally.refused += len(lost)
    # #1010: refuse before the run, run the cases in chain order, then print in file order.
    stopped: set[int] = set()
    plan: list[tuple[Shape, list[Case[Any]], bool]] = []
    faults_of: dict[int, list[str]] = {}
    for test, shape in shapes.items():
        mine = sorted((c for c in cases if c.test is test and id(c) in names), key=lambda c: c.line)
        plan.append((shape, mine, bool(mine)))
        if shape.faults:
            stopped |= {id(c) for c in mine}
            continue
        for c in mine:
            if faults := refuse_case(c, shape, src, scope, shapes, written=True):
                faults_of[id(c)] = faults
                stopped.add(id(c))
    queue: list[tuple[str, Any]] = []
    runnable: list[Case[Any]] = []
    for shape, mine, written in plan:
        queue.append(("say", f" {shape.name}  ({rel(path)}:{shape.node.lineno})"))
        if shape.faults:
            queue.append(("say", "   ! refused:\n       " + "\n       ".join(shape.faults)))
            queue.append(("say", f"     so none of its {max(len(mine), 1)} case(s) runs"))
            tally.refused += max(len(mine), 1)
            continue
        needs = [n for n, p in shape.params.items() if p.default is inspect.Parameter.empty]
        if not mine and needs:
            text = (
                f"   ! refused: it takes {', '.join(needs)}, and no case gives them: "
                f"add `name = case({shape.name}, …)`"
            )
            queue.append(("say", text))
            tally.refused += 1
            continue
        lone = Case(shape.fn, (), {}, str(path), shape.node.lineno) if not mine else None
        run_cases = [(names[id(c)], c) for c in mine] or [(shape.name, lone)]
        for name, case in run_cases:
            assert case is not None
            where = f"{rel(case.file)}:{case.line}"
            if not written:
                if faults := refuse_case(case, shape, src, scope, shapes, written=False):
                    faults_of[id(case)] = faults
            if id(case) in faults_of:
                listed = "\n       ".join(faults_of[id(case)])
                queue.append(("say", f"   ! {name} refused   {where}\n       {listed}"))
                tally.refused += 1
                stopped.add(id(case))
                continue
            queue.append(("case", (name, case, written, where)))
            runnable.append(case)
    # a case whose chain holds a case refused before the run is blocked before the run,
    # however far up that case is
    gone_of: dict[int, str] = {}
    for case in order_of(runnable):
        earlier = [v.case for v in case.inputs.values() if isinstance(v, Step)]
        gone = next((names.get(id(e), "?") for e in earlier if id(e) in stopped), None)
        gone = gone or next((gone_of[id(e)] for e in earlier if id(e) in gone_of), None)
        if gone:
            gone_of[id(case)] = gone
            stopped.add(id(case))
    runnable = [c for c in runnable if id(c) not in gone_of]
    fell_back = run_chains(runnable, shapes, names)
    for kind_, item in queue:
        if kind_ == "say":
            say(item)
            continue
        name, case, written, where = item
        if id(case) in gone_of:
            gone = gone_of[id(case)]
            say(f"   ⊘ {name}  blocked: it takes the product of {gone}, which is refused")
            tally.blocked += 1
            continue
        if id(case) in fell_back:
            say(f"   ↺ {name} ran by replay: {fell_back[id(case)]}")
        results, captured = read_case(case)
        report(name, case if written else None, results, captured, tally, where)


def steps_of(case: Case[Any], among: set[int]) -> list[Case[Any]]:
    """The earlier cases this case takes a product of, in the order of its inputs, once each."""
    out: dict[int, Case[Any]] = {}
    for v in case.inputs.values():
        if isinstance(v, Step) and id(v.case) in among:
            out.setdefault(id(v.case), v.case)
    return list(out.values())


def order_of(cases: list[Case[Any]]) -> list[Case[Any]]:
    """Each case after the cases it takes a product of, and otherwise in file order."""
    among, done, out = {id(c) for c in cases}, set[int](), list[Case[Any]]()
    left_ = list(cases)
    while left_:
        ready = next((c for c in left_ if all(id(s) in done for s in steps_of(c, among))), left_[0])
        left_.remove(ready)
        done.add(id(ready))
        out.append(ready)
    return out


def run_chains(
    cases: list[Case[Any]], shapes: dict[Any, Shape], names: dict[int, str]
) -> dict[int, str]:
    """Run each case into its files. Replay: one process for each case, in chain order. Tree: one
    tree for each case that takes no product, whose later stages fork from it; a case with two
    earlier stages forks from the first and replays the others. Returns each case that the tree
    fell back to replay for, with the reason."""
    order = order_of(cases)
    if CHAIN[0] == "replay":
        for case in order:
            run_case(case, Run(shapes, names))
        return {}
    among = {id(c) for c in cases}
    first = {id(c): (steps_of(c, among) or [None])[0] for c in order}
    tree: dict[int, list[Case[Any]]] = {}
    for c in order:
        if (p := first[id(c)]) is not None:
            tree.setdefault(id(p), []).append(c)

    def root_of(c: Case[Any]) -> Case[Any]:
        while (p := first[id(c)]) is not None:
            c = p
        return c

    def subtree(c: Case[Any]) -> list[Case[Any]]:
        return [c] + [d for k in tree.get(id(c), []) for d in subtree(k)]

    roots = [c for c in order if first[id(c)] is None]
    needs = {
        id(r): {id(root_of(s)) for c in subtree(r) for s in steps_of(c, among)[1:]} - {id(r)}
        for r in roots
    }
    ran: set[int] = set()
    fell_back: dict[int, str] = {}
    # a tree runs after the trees of its cases' other earlier stages: the refused rule reads them
    while roots:
        root = next((r for r in roots if needs[id(r)] <= ran), roots[0])
        roots.remove(root)
        ran.add(id(root))
        run_case(root, Run(shapes, names, tree=tree, kids=list(tree.get(id(root), []))))
        members = {id(c) for c in subtree(root)}
        for c in order:  # a stage that fell back left its later stages: replay them in order
            if id(c) in members and results_of(c) is None:
                run_case(c, Run(shapes, names))
                up = c
                while (up := first[id(up)]) is not None and not os.path.exists(job(up, "fallback")):
                    pass
                why = "its earlier stage left no result"
                fell_back[id(c)] = Path(job(up, "fallback")).read_text() if up else why
    return fell_back


def main(argv: list[str]) -> int:
    if "--tree" in argv:
        argv = [x for x in argv if x != "--tree"]
        CHAIN[0] = "tree"
    JOBS.append(tempfile.mkdtemp(prefix="p1010-"))
    tally = Tally()
    for path in files_in(argv):
        run_file(path, tally)
    total = tally.passed + tally.failed + tally.refused + tally.blocked
    files = f" · {tally.files} file{'s' * (tally.files != 1)} refused" if tally.files else ""
    say(f"══ {total} case{'s' * (total != 1)}: {tally.passed} passed · {tally.failed} failed · {tally.refused} refused · {tally.blocked} blocked{files}   ({tally.checks} checks held)")
    shutil.rmtree(JOBS[0])
    return 0 if total and total == tally.passed and not tally.files else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
