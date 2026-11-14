"""Run the tests in files or folders, and print what each case shows. Prototype of #979, on prototype 5 of #967. Never merge.

#979 adds: any `with` in the act is a setup; the act ends at the first check, so checks sit at the end of a `with`,
after it, or both; a setup that swallows a raise and skips a line fails the case, and so does an undo that raises;
the act may not replace a hook the runner sees faults through; the parent prints through a stream of its own.

    python run.py [path ...]     each path a file or a folder; the default is the current folder

A test is a function named `test…` in a file named `test_….py`. The runner reads its source, refuses what breaks the
form, and imports the file with each test's last asserts rewritten as checks, as pytest rewrites an assert.
Then it runs each test as written, once for each case, in a process of its own; each check but the last runs in a
fork of that process. The exit status is 0 when every case passes, and 1 when a case fails, is refused or is blocked.
"""

from __future__ import annotations

import ast
import builtins
import difflib
import importlib.util
import inspect
import json
import math
import mmap
import operator
import os
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

from cases import BEFORE, CREATED, OWNED, SEEN, Case, Checkpoints, Missed, Samples, Step, Stock, Use  # noqa: E402

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

ONE_PATH = "the act has one path"
LOGIC: dict[type, str] = {
    ast.If: "an `if`", ast.IfExp: "an `x if c else y`", ast.BoolOp: "an `and` or `or`", ast.Match: "a `match`",
    ast.While: "a `while`", ast.Break: "a `break`", ast.Continue: "a `continue`",
    ast.Try: "a `try`: expect a raise with `with raises(T):`, or run code inside the `except` with `with handling(T, call, …):`",
    ast.TryStar: "a `try`: expect a raise with `with raises(T):`",
    ast.With: "a `with` inside a loop, a class or a function: a setup wraps statements of the act itself",
    ast.AsyncWith: "an `async with`",
    ast.Assert: "an assert inside the act: the act ends at the first check, and every check comes after it, at the end of the test or at the end of a `with`",
    ast.Raise: "a `raise`: expect the code's own raise with `with raises(…):`",
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
    return f"computes `{ast.unparse(node)}`; an expected value is a literal, so the test cannot compute what the code computes"


def fixed_data(node: ast.expr, src: Source) -> bool:
    """Data a loop may run over and still have one path: a literal, or `range(…)` of literals."""
    if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == "range" and not node.keywords:
        return all(literal_or_why(a, src) is None for a in node.args)
    return literal_or_why(node, src) is None


Own = tuple[set[int], set[str]]  # the functions, lambdas and classes a test runs itself; the methods it calls by name


def called_inside(test: ast.FunctionDef) -> Own:
    """What the test runs itself: its body is part of the act. A function, lambda or class whose only use is as an
    argument of a call is a value the test hands to the code. Handing one to `map` or `filter` is a loop of the test's own."""
    parent = {id(c): p for p in ast.walk(test) for c in ast.iter_child_nodes(p)}

    def handed(node: ast.expr) -> bool:
        p = parent.get(id(node))
        if isinstance(p, ast.keyword):
            p = parent.get(id(p))
        elif not (isinstance(p, ast.Call) and node in p.args):
            return False
        return isinstance(p, ast.Call) and not (isinstance(p.func, ast.Name) and p.func.id in {"map", "filter"})

    named: dict[str, ast.AST] = {}
    for n in ast.walk(test):
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)) and n is not test:
            named[n.name] = n
        elif isinstance(n, ast.Assign) and isinstance(n.value, ast.Lambda) and len(n.targets) == 1 and isinstance(n.targets[0], ast.Name):
            named[n.targets[0].id] = n.value
    own = {id(named[n.id]) for n in ast.walk(test) if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Load) and n.id in named and not handed(n)}
    own |= {id(n) for n in ast.walk(test) if isinstance(n, ast.Lambda) and id(n) not in map(id, named.values()) and not handed(n)}
    methods = {n.func.attr for n in ast.walk(test) if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute)}
    return own, methods


def logic_in(node: ast.AST, src: Source, own: Own) -> list[str]:
    """Each place where the act would have more than one path."""
    faults: list[str] = []
    ids, methods = own

    def value(n: ast.AST) -> None:  # a value the test hands to the code: its body is the code's to run, not the act
        for inner in ast.walk(n):
            if isinstance(inner, ast.Assert):
                faults.append(f"line {inner.lineno} holds an assert inside a function the test hands to the code, which may swallow it: keep what it sees, and check that at the end")

    def visit(n: ast.AST) -> None:
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)) and id(n) not in ids:
            return value(n)
        if isinstance(n, ast.ClassDef):
            if id(n) not in ids:
                return value(n)
            for stmt in n.body:  # the test builds it: its body and `__init__` run in the act, and so does a method it calls
                runs = not isinstance(stmt, (ast.FunctionDef, ast.AsyncFunctionDef)) or stmt.name in {"__init__", "__new__", *methods}
                ids.add(id(stmt)) if runs and not isinstance(stmt, ast.ClassDef) else None
                visit(stmt)
            return
        if isinstance(n, (ast.For, ast.AsyncFor)):
            if isinstance(n, ast.AsyncFor) or n.orelse or not fixed_data(n.iter, src):
                faults.append(f"line {n.lineno} holds a loop over `{ast.unparse(n.iter)}`: {ONE_PATH}, so a loop runs over literal data, such as `range(5)`, with no `else`; or make each round a case")
            for stmt in n.body:
                visit(stmt)
            return
        if isinstance(n, (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
            for gen in n.generators:
                if gen.ifs or gen.is_async or not fixed_data(gen.iter, src):
                    faults.append(f"line {n.lineno} holds a comprehension over `{ast.unparse(gen.iter)}`" + (" with an `if`" if gen.ifs else "") + f": {ONE_PATH}, so it runs over literal data, with no `if`")
            for child in ast.iter_child_nodes(n):
                visit(child)
            return
        if type(n) in LOGIC:
            faults.append(f"line {getattr(n, 'lineno', '?')} holds {LOGIC[type(n)]}" + (f": {ONE_PATH}" if type(n) in (ast.If, ast.IfExp, ast.BoolOp, ast.Match, ast.While, ast.Break, ast.Continue) else ""))
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


def read_with(stmt: ast.With, last: bool, shape: Shape, src: Source, called: Own, walk: Callable[[list[ast.stmt], bool], None]) -> None:
    if is_setup(stmt):  # a setup: what it sets up holds for the statements inside it, and ends with them (#979)
        for item in stmt.items:
            shape.faults += logic_in(item.context_expr, src, called)
        walk(stmt.body, last)
        return
    kind = expecter(stmt.items[0]) if len(stmt.items) == 1 else None
    if kind is None:
        shape.faults.append(f"line {stmt.lineno}: `raises`, `warns` and `handling` each take a `with` of their own")
        return
    if len(stmt.body) != 1:
        shape.faults.append(f"line {stmt.lineno}: `with {kind}(…):` holds one statement, the one that raises or warns; it holds {len(stmt.body)}")
    if kind == "raises" and not last:
        shape.faults.append(f"line {stmt.lineno}: `with raises(…):` is the last statement of the act; after a raise the act has nothing left to run")
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
        shape.faults += logic_in(inner, src, called)
    if kind == "raises":
        shape.made_in_raises |= stored(stmt.body)


def read_check(stmt: ast.Assert, shape: Shape, src: Source, called: Own) -> Check | None:
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
    for fault in logic_in(reading, src, called):
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
    called = called_inside(node)
    is_check = {id(a) for a in asserts}

    def walk(stmts: list[ast.stmt], last: bool) -> None:
        stmts = [s for s in stmts if id(s) not in is_check]
        for i, stmt in enumerate(stmts):
            if isinstance(stmt, ast.Return):
                shape.faults.append(f"line {stmt.lineno} returns before the checks: `return` is the last line of a test")
            elif isinstance(stmt, ast.With):
                read_with(stmt, last and i == len(stmts) - 1, shape, src, called, walk)
            else:
                shape.faults += logic_in(stmt, src, called)

    walk(act, True)
    for stmt in asserts:
        if check := read_check(stmt, shape, src, called):
            shape.checks.append(check)
    shape.returns = ret.value if ret else None
    if shape.returns is not None:
        shape.faults += logic_in(shape.returns, src, called)
        if any(e.startswith("raises") for e in shape.expects):
            shape.faults.append(f"line {shape.returns.lineno}: a test that expects a raise makes no product, so it returns nothing")
    if not shape.checks and not shape.expects and not shape.faults:
        shape.faults.append("checks nothing: end it with `assert reading == expected`, or expect a raise with `with raises(…):`")
    for name, p in shape.params.items():
        if isinstance(p.default, Use) and isinstance(p.default.factory, type) and issubclass(p.default.factory, Stock):
            stock = p.default.factory
            if inspect.isabstract(stock):
                missing = ", ".join(sorted(getattr(stock, "__abstractmethods__", ())))
                shape.faults.append(f"the input {name}: the stock {stock.__name__} lacks {missing}; a misspelled method leaves it abstract")
            elif not issubclass(stock, (Samples, Checkpoints)):
                shape.faults.append(f"the input {name}: the stock {stock.__name__} offers neither Samples nor Checkpoints, so nothing seals it")
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


def refuse_unnamed(case: Case[Any], src: Source) -> str:
    stmt = src.statement_at(case.line)
    if isinstance(stmt, (ast.For, ast.While)) or (stmt and any(isinstance(n, (ast.ListComp, ast.GeneratorExp, ast.DictComp)) for n in ast.walk(stmt))):
        return "is built by a loop; write each case on a line of its own, `name = case(test, …)`"
    if isinstance(stmt, ast.Assign) and len(stmt.targets) == 1 and isinstance(stmt.targets[0], ast.Name):
        name = stmt.targets[0].id
        later = [n for n in src.bindings.get(name, []) if n > stmt.lineno]
        if later:
            return f"is named {name!r}, and line {later[0]} binds {name!r} again, so this case would be lost"
    return "has no name; bind it to a variable, `name = case(test, …)`"


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


# ── ③ RUN: one forked process for each case; each check but the last in a fork of its own ──


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
    os.write(fd, (json.dumps(result) + "\n").encode())


def frames(error: BaseException, file: str) -> list[str]:
    shown = [f for f in traceback.extract_tb(error.__traceback__) if os.path.realpath(f.filename) == os.path.realpath(file)]
    return [f"at {rel(f.filename)}:{f.lineno}  {f.line}" for f in shown[-2:]]


def check_one(run: Run, shape: Shape, check: Check, pair: Callable[[], Any], message: Callable[[], Any]) -> None:
    """Read one value and compare it, in this process: only the verdict, and the two texts of a failure, cross back."""
    try:
        left, right = pair()
        asks = message()
    except Exception as error:
        emit(run.fd, check=check.label, line=check.line, status="fail", why=f"reading it raised {type(error).__name__}: {error}", trace=frames(error, shape.file))
        return
    why = verdict(check, left, right)
    reading, expected = (left, right) if check.reading_left else (right, left)
    if why is None:
        emit(run.fd, check=check.label, line=check.line, status="pass", reading=short(reading))
        return
    if not why and short(expected) == short(reading):
        why = f"same text, types {kind(expected)} and {kind(reading)}"
    emit(run.fd, check=check.label, line=check.line, status="fail", asks=asks, expected=f"{short(expected)} ({kind(expected)})", reading=f"{short(reading)} ({kind(reading)})", why=why)


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
    if run.replaying:
        left, right = pair()
        if (why := verdict(check, left, right)) is not None:
            reading, expected = (left, right) if check.reading_left else (right, left)
            raise Blocked(f"{run.replaying[-1]} › {check.label} diverged on replay: expected {short(expected)}, read {short(reading)}" + (f" ({why})" if why else ""))
        return
    run.done.add(k)
    if run.guard:
        run.guard.verify()
    local = caller.f_locals
    readers = lambda n: sum(n in c.names for c in shape.checks)  # noqa: E731
    held = sorted(n for n in check.names if n in local and is_handle(local[n]) and readers(n) >= 2)
    if held:
        emit(run.fd, check=check.label, line=check.line, status="refused", why=f"{held[0]} is an open file, socket, pipe or mmap, and more than one check reads it: a fork cannot seal it")
        return
    stocks = [n for n in check.names if n in run.stock_inputs]
    if stocks:  # a check of a stock runs in place, between a checkpoint and a restore (#965)
        stock, value = run.stock_inputs[stocks[0]], local.get(stocks[0])
        if not isinstance(stock, Checkpoints) and readers(stocks[0]) >= 2:
            emit(run.fd, check=check.label, line=check.line, status="refused", why=f"{stocks[0]} is a stock with no checkpoint, and more than one check reads it")
            return
        mark = stock.checkpoint(value) if isinstance(stock, Checkpoints) else None
        check_one(run, shape, check, pair, message)
        if isinstance(stock, Checkpoints):
            stock.restore(value, mark)
        return
    if k == len(shape.checks) - 1:  # the last check runs in place: nothing reads after it
        check_one(run, shape, check, pair, message)
        return
    flush()
    pid = os.fork()  # each check but the last in a fork of its own: none can change what another reads (#964)
    if pid == 0:
        check_one(run, shape, check, pair, message)
        os._exit(0)
    os.waitpid(pid, 0)


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
    shape = run.shapes[case.test]
    lost: list[str] = []
    sys.unraisablehook = lambda u: lost.append(f"{type(u.exc_value).__name__}: {u.exc_value}")
    threading.excepthook = lambda h: lost.append(f"in a thread, {h.exc_type.__name__}: {h.exc_value}")

    def block_the_rest(why: str) -> None:
        for k, c in enumerate(shape.checks):
            if k not in run.done:
                emit(run.fd, check=c.label, line=c.line, status="blocked", why=why)

    try:
        for file, fn in BEFORE:
            if os.path.realpath(file) == os.path.realpath(shape.file):
                try:
                    fn()
                except Exception as error:
                    emit(run.fd, check=f"before_each({name_of(fn)})", line=0, status="fail", why=f"it raised {type(error).__name__}: {error}")
                    return block_the_rest("the case did not start")
        try:
            inputs = build(case, run, top=True)
        except Blocked as why:
            return block_the_rest(str(why))
        except Exception as error:
            emit(run.fd, check="inputs", line=0, status="fail", why=f"building an input raised {type(error).__name__}: {error}", trace=frames(error, shape.file))
            return block_the_rest("an input was not built")
        for name, value in inputs.items():  # a stand-in matches the type that the test declares
            allowed = annotation_types(shape.params[name].annotation, vars(sys.modules[case.test.__module__]))
            if allowed and isinstance(shape.params[name].default, Use) and name not in case.inputs and not fits(value, allowed):
                emit(run.fd, check="inputs", line=0, status="fail", why=f"the fixture of {name} gives {a(kind(value))}, and the test declares {' | '.join(t.__name__ for t in allowed)}")
                return block_the_rest("an input was not built")
        SEEN.clear()
        error: BaseException | None = None
        guard = run.guard = Guard(run)
        guard.install()
        with warnings.catch_warnings(record=True) as seen:
            warnings.simplefilter("always")
            guard.start()
            try:
                case.test(**inputs)
            except (Missed, Exception) as raised:
                error = raised
            finally:
                guard.verify()
                guard.on = False
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


def run_case(case: Case[Any], run: Run) -> tuple[list[dict[str, Any]], str]:
    flush()
    r, w = os.pipe()
    out = tempfile.TemporaryFile()
    pid = os.fork()
    if pid == 0:
        os.close(r)
        os.dup2(out.fileno(), 1)
        os.dup2(out.fileno(), 2)
        run.fd = w
        try:
            in_case(case, run)
        except BaseException as error:  # a fault of the runner, shown as it is
            emit(w, check="runner", line=0, status="fail", why=f"{type(error).__name__}: {error}")
        flush()
        os._exit(0)
    os.close(w)
    data = b""
    while chunk := os.read(r, 65536):
        data += chunk
    os.close(r)
    _, status = os.waitpid(pid, 0)
    out.seek(0)
    captured = out.read().decode(errors="replace")
    results = [json.loads(line) for line in data.decode().splitlines()]
    if os.WIFSIGNALED(status):
        results.append({"check": "act", "line": 0, "status": "fail", "why": f"the case's process died: signal {os.WTERMSIG(status)} (a case gets {SECONDS} s)"})
    return results, captured


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
        status = "fail"
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
    names = {id(v): k for k, v in scope.items() if isinstance(v, Case)}
    shapes = {t: shape_of(t, src) for t in tests}
    unnamed: dict[int, list[Case[Any]]] = {}
    for c in cases:
        if id(c) not in names:
            unnamed.setdefault(c.line, []).append(c)
        elif c.test not in shapes:
            say(f"   ! {names[id(c)]} at line {c.line} is a case of {name_of(c.test)}, which is no test of this file: a test is a function named `test…`")
            tally.refused += 1
    for line, lost in unnamed.items():
        count = f"{len(lost)} cases" if len(lost) > 1 else "a case"
        why = refuse_unnamed(lost[0], src)
        say(f"   ! {count} of {name_of(lost[0].test)} at line {line} {why.replace('is built', 'are built', 1) if len(lost) > 1 else why}")
        tally.refused += len(lost)
    stopped: set[int] = set()
    for test, shape in shapes.items():
        mine = sorted((c for c in cases if c.test is test and id(c) in names), key=lambda c: c.line)
        say(f" {shape.name}  ({rel(path)}:{shape.node.lineno})")
        if shape.faults:
            say("   ! refused:\n       " + "\n       ".join(shape.faults))
            say(f"     so none of its {max(len(mine), 1)} case(s) runs")
            tally.refused += max(len(mine), 1)
            stopped |= {id(c) for c in mine}
            continue
        needs = [n for n, p in shape.params.items() if p.default is inspect.Parameter.empty]
        if not mine and needs:
            say(f"   ! refused: it takes {', '.join(needs)}, and no case gives them: add `name = case({shape.name}, …)`")
            tally.refused += 1
            continue
        run_cases: list[tuple[str, Case[Any]]] = [(names[id(c)], c) for c in mine] or [(shape.name, Case(test, (), {}, str(path), shape.node.lineno))]
        for name, case in run_cases:
            faults = refuse_case(case, shape, src, scope, shapes, written=bool(mine))
            where = f"{rel(case.file)}:{case.line}"
            if faults:
                say(f"   ! {name} refused   {where}\n       " + "\n       ".join(faults))
                tally.refused += 1
                stopped.add(id(case))
                continue
            gone = next((names.get(id(v.case), "?") for v in case.inputs.values() if isinstance(v, Step) and id(v.case) in stopped), None)
            if gone:
                say(f"   ⊘ {name}  blocked: it takes the product of {gone}, which is refused")
                tally.blocked += 1
                continue
            results, captured = run_case(case, Run(shapes, names))
            report(name, case if mine else None, results, captured, tally, where)


def main(argv: list[str]) -> int:
    tally = Tally()
    for path in files_in(argv):
        run_file(path, tally)
    total = tally.passed + tally.failed + tally.refused + tally.blocked
    files = f" · {tally.files} file{'s' * (tally.files != 1)} refused" if tally.files else ""
    say(f"══ {total} case{'s' * (total != 1)}: {tally.passed} passed · {tally.failed} failed · {tally.refused} refused · {tally.blocked} blocked{files}   ({tally.checks} checks held)")
    return 0 if total and total == tally.passed and not tally.files else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
