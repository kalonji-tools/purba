"""Run the tests in files or folders, and print what each case shows. Prototype 3 (#967). Never merge.

    python run.py [path ...]     each path a file or a folder; the default is the current folder

A test is a function named `test…` in a file named `test_….py`. The runner reads its source and runs it in parts:
the statements before its last asserts are the act, and each of its last asserts is a check of its own.
The exit status is 0 when every case passes, and 1 when a case fails, is refused or is blocked.
"""

from __future__ import annotations

import ast
import builtins
import copy
import difflib
import importlib
import importlib.util
import inspect
import json
import math
import mmap
import os
import re
import signal
import sys
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

from cases import CREATED, SEEN, Case, Checkpoints, Missed, Samples, Step, Stock, Use  # noqa: E402

LIBRARY = {"run.py", "cases.py"}
SECONDS = 30
LONG = 400

# ── ① PARSE: read each test, and refuse before anything runs ────────────────────────

LOOP = "a loop: write each call on a line of its own, or make each round a case"
BRANCH = "a branch: make each side a case of its own"
LOGIC: dict[type, str] = {
    ast.If: BRANCH, ast.IfExp: BRANCH, ast.BoolOp: "an `and` or `or` picks which side runs: make each side a case of its own",
    ast.Match: BRANCH, ast.For: LOOP, ast.AsyncFor: LOOP, ast.While: LOOP,
    ast.ListComp: LOOP, ast.SetComp: LOOP, ast.DictComp: LOOP, ast.GeneratorExp: LOOP,
    ast.Try: "a `try` inside a function of the test: write it in the test itself",
    ast.TryStar: "an `except*`: write one `except Type:`",
    ast.With: "a `with` other than `raises` or `warns`: call what it sets up, or make it a fixture with use(…)",
    ast.AsyncWith: "an `async with`: call what it sets up, or make it a fixture with use(…)",
    ast.Assert: "an assert inside a function of the test: check at the end of the test",
    ast.Raise: "a `raise`: expect the code's own raise with `with raises(…):`",
    ast.Global: "a `global`: take the value as an input", ast.Nonlocal: "a `nonlocal`: take the value as an input",
    ast.Yield: "a `yield`: a test returns its product", ast.YieldFrom: "a `yield`: a test returns its product",
    ast.Await: "an `await`: this form runs no coroutine",
}
EXPECTERS = {"raises", "warns"}


@dataclass
class Check:
    """One of a test's last asserts: it reads a value, and the runner compares the reading with what the case expects."""

    node: ast.Assert
    reading: ast.expr
    expected: ast.expr
    message: ast.expr | None
    names: set[str]

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
    act: list[ast.stmt] = field(default_factory=list)
    checks: list[Check] = field(default_factory=list)
    returns: ast.expr | None = None
    expects: list[str] = field(default_factory=list)  # `raises` and `warns`, as written
    expected_inputs: set[str] = field(default_factory=set)  # inputs that a case fills with an expected value
    kind_inputs: set[str] = field(default_factory=set)  # inputs that name an exception or a warning type
    made: set[str] = field(default_factory=set)  # names the act assigns
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


def name_of(fn: object) -> str:
    return str(getattr(fn, "__name__", fn))


def hint(word: str, choices: list[str]) -> str:
    close = difflib.get_close_matches(word, choices, n=1)
    return f"; did you mean {close[0]!r}?" if close else f"; it has {', '.join(map(repr, choices)) or 'none'}"


def logic_in(node: ast.AST) -> list[str]:
    return [f"line {getattr(n, 'lineno', '?')} holds {LOGIC[type(n)]}" for n in ast.walk(node) if type(n) in LOGIC]


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


def expected_or_why(node: ast.expr, shape: Shape, src: Source) -> str | None:
    """An expected value is a literal, an input of the test, or a list, tuple, set or dict of these."""
    if isinstance(node, ast.Name) and node.id in shape.params:
        shape.expected_inputs.add(node.id)
        return None
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        return next((w for w in (expected_or_why(e, shape, src) for e in node.elts) if w), None)
    if isinstance(node, ast.Dict):
        return next((w for w in (expected_or_why(e, shape, src) for e in [*node.keys, *node.values] if e is not None) if w), None)
    return literal_or_why(node, src)


def stored(node: ast.AST) -> set[str]:
    names = {n.id for n in ast.walk(node) if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Store)}
    names |= {n.name for n in ast.walk(node) if isinstance(n, (ast.FunctionDef, ast.ClassDef))}
    return names | {h.name for n in ast.walk(node) if isinstance(n, ast.Try) for h in n.handlers if h.name}


def expecter(item: ast.withitem) -> str | None:
    call = item.context_expr
    if isinstance(call, ast.Call):
        fn = call.func
        name = fn.id if isinstance(fn, ast.Name) else fn.attr if isinstance(fn, ast.Attribute) else None
        if name in EXPECTERS:
            return name
    return None


def read_with(stmt: ast.With, last: bool, shape: Shape, src: Source) -> None:
    kinds = [expecter(i) for i in stmt.items]
    if len(stmt.items) != 1 or kinds[0] is None:
        shape.faults.append(f"line {stmt.lineno} holds {LOGIC[ast.With]}")
        return
    kind = kinds[0]
    if len(stmt.body) != 1:
        shape.faults.append(f"line {stmt.lineno}: `with {kind}(…):` holds one statement, the one that {kind[:-1]}s; it holds {len(stmt.body)}")
    if kind == "raises" and not last:
        shape.faults.append(f"line {stmt.lineno}: `with raises(…):` is the last statement of the act; after a raise the act has nothing left to run")
    if stmt.items[0].optional_vars is not None:
        shape.faults.append(f"line {stmt.lineno}: `with {kind}(…) as …`: the runner prints what it caught; drop the `as`")
    call = stmt.items[0].context_expr
    assert isinstance(call, ast.Call)
    params = shape.params
    for i, arg in enumerate([*call.args, *(k.value for k in call.keywords)]):
        if isinstance(arg, ast.Name) and arg.id in params:
            (shape.kind_inputs if i == 0 and call.args else shape.expected_inputs).add(arg.id)
        elif i == 0 and call.args and isinstance(arg, (ast.Name, ast.Attribute)):
            pass  # the type it expects, by name
        elif (why := literal_or_why(arg, src)) is not None:
            shape.faults.append(f"line {stmt.lineno}: `{kind}(…)` {why}")
    shape.expects.append(ast.unparse(call))
    for inner in stmt.body:
        shape.faults += logic_in(inner)
    if kind == "raises":
        shape.made_in_raises |= stored(stmt)


def read_try(stmt: ast.Try, shape: Shape) -> None:
    form = "a `try` runs its `except` only when its body raises: write one statement in the body, one `except Type:`, and no `else` or `finally`"
    handler = stmt.handlers[0] if len(stmt.handlers) == 1 else None
    if len(stmt.body) != 1 or handler is None or handler.type is None or isinstance(handler.type, ast.Tuple) or stmt.orelse or stmt.finalbody:
        shape.faults.append(f"line {stmt.lineno}: {form}")
        return
    for inner in [*stmt.body, *handler.body]:
        shape.faults += logic_in(inner)


def read_check(stmt: ast.Assert, shape: Shape, src: Source) -> Check | None:
    test = stmt.test
    where = f"line {stmt.lineno}"
    if not isinstance(test, ast.Compare):
        shape.faults.append(f"{where}: an assert compares a reading with the value you expect: `assert reading == expected`")
        return None
    if len(test.ops) != 1 or not isinstance(test.ops[0], ast.Eq):
        shape.faults.append(f"{where}: compare with `==`, and write the value you expect; the runner compares by type, then by value")
        return None
    reading, expected = test.left, test.comparators[0]
    if literal_or_why(reading, src) is None and literal_or_why(expected, src) is not None:
        shape.faults.append(f"{where}: the reading goes on the left, and the expected value on the right")
        return None
    if (why := expected_or_why(expected, shape, src)) is not None:
        shape.faults.append(f"{where}: the expected value {why}; write a literal, or take it as an input of the case")
    for fault in logic_in(reading):
        shape.faults.append(f"{where}: the reading {fault.split(' holds ', 1)[1]}" if " holds " in fault else fault)
    names = {n.id for n in ast.walk(reading) if isinstance(n, ast.Name)}
    if not names:
        shape.faults.append(f"{where}: the reading `{ast.unparse(reading)}` reads no value; read what the act made")
    message = stmt.msg
    if message is not None and not (isinstance(message, ast.JoinedStr) or (isinstance(message, ast.Constant) and isinstance(message.value, str))):
        shape.faults.append(f"{where}: the message of an assert is a string, or an f-string")
    late = sorted(names & shape.made_in_raises)
    if late:
        shape.faults.append(f"{where} reads {', '.join(late)}, which the act assigns inside `with raises(…):`, so it never exists")
    return Check(stmt, reading, expected, message, names)


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
    body = node.body
    if body and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant) and isinstance(body[0].value.value, str):
        body = body[1:]
    if body and isinstance(body[-1], ast.Return):
        shape.returns = body[-1].value
        body = body[:-1]
    split = len(body)
    while split > 0 and isinstance(body[split - 1], ast.Assert):
        split -= 1
    shape.act = body[:split]
    for i, stmt in enumerate(shape.act):
        last = i == len(shape.act) - 1
        if isinstance(stmt, ast.Assert):
            shape.faults.append(f"line {stmt.lineno} holds an assert before the act ends: move every assert to the end of the test, or make the steps before it a case of its own")
        elif isinstance(stmt, ast.Return):
            shape.faults.append(f"line {stmt.lineno} returns before the checks: `return` is the last line of a test")
        elif isinstance(stmt, ast.Try):
            read_try(stmt, shape)
        elif isinstance(stmt, ast.With):
            read_with(stmt, last, shape, src)
        else:
            shape.faults += logic_in(stmt)
    shape.made = stored(ast.Module(body=shape.act, type_ignores=[]))
    for stmt in body[split:]:
        assert isinstance(stmt, ast.Assert)
        check = read_check(stmt, shape, src)
        if check:
            shape.checks.append(check)
    if shape.returns is not None:
        shape.faults += logic_in(shape.returns)
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
    for check in shape.checks:  # an assay reads the product, or an input that the act can change (#965)
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


# ── ②-④ BUILD, ACT, CHECK: one forked process for each case ────────────────────────


class Blocked(Exception):
    pass


class TryNotRaised(Exception):
    pass


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


def compile_act(shape: Shape, module: types.ModuleType) -> Callable[..., dict[str, Any]]:
    """The test's act as a function of its own: the same lines, each `try` proved, and it returns its names."""
    node = copy.deepcopy(shape.node)
    body: list[ast.stmt] = []
    for stmt in copy.deepcopy(shape.act):
        if isinstance(stmt, ast.Try):  # the body must raise, so the `except` runs: else the case fails
            what = ast.unparse(stmt.handlers[0].type) if stmt.handlers[0].type else "?"
            stmt.orelse = [ast.Raise(ast.Call(ast.Name("__try_not_raised__", ast.Load()), [ast.Constant(f"line {stmt.lineno}: the body of the `try` did not raise {what}, so its `except` never ran")], []))]
        body.append(stmt)
    body.append(ast.Return(ast.Call(ast.Name("locals", ast.Load()), [], [])))
    node.body, node.decorator_list, node.returns = body, [], None
    for arg in [*node.args.posonlyargs, *node.args.args, *node.args.kwonlyargs]:
        arg.annotation = None
    node.args.defaults, node.args.kw_defaults = [], [None] * len(node.args.kwonlyargs)
    tree = ast.fix_missing_locations(ast.Module(body=[node], type_ignores=[]))
    scope = vars(module)
    scope["__try_not_raised__"] = TryNotRaised
    test = scope.get(shape.name)
    exec(compile(tree, shape.file, "exec"), scope)  # the act keeps the test's name, which a library may record
    act = scope[shape.name]
    scope[shape.name] = test
    return act


def evaluate(node: ast.expr, shape: Shape, module: types.ModuleType, ns: dict[str, Any]) -> Any:
    return eval(compile(ast.Expression(node), shape.file, "eval"), vars(module), ns)


@dataclass
class Run:
    """What one case's process holds: the products of replayed steps, and each stock it drew."""

    shapes: dict[Any, Shape]
    names: dict[int, str]
    memo: dict[int, object] = field(default_factory=dict)
    stocks: dict[type, tuple[Stock[Any], object, object]] = field(default_factory=dict)  # stock type → (stock, value, checkpoint)
    stock_inputs: dict[str, Stock[Any]] = field(default_factory=dict)


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


def module_of(fn: Callable[..., Any]) -> types.ModuleType:
    return sys.modules[fn.__module__]


def replay(step: Case[Any], run: Run) -> object:
    """Run an earlier case again, once for each case: its act, then its checks. A step that reads differently blocks the case."""
    if id(step) in run.memo:
        return run.memo[id(step)]
    shape = run.shapes[step.test]
    module = module_of(step.test)
    label = run.names.get(id(step), f"the case at line {step.line}")
    inputs = build(step, run, top=False)
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        try:
            ns = compile_act(shape, module)(**inputs)
        except Exception as error:
            raise Blocked(f"the step {label} raised {type(error).__name__}: {error}") from None
    for check in shape.checks:
        reading, expected = evaluate(check.reading, shape, module, ns), evaluate(check.expected, shape, module, ns)
        if differs(expected, reading) is not None:
            raise Blocked(f"{label} › {check.label} diverged on replay: expected {short(expected)}, read {short(reading)}")
    product = evaluate(shape.returns, shape, module, ns) if shape.returns is not None else None
    run.memo[id(step)] = product
    return product


def emit(fd: int, **result: object) -> None:
    os.write(fd, (json.dumps(result) + "\n").encode())


def frames(error: BaseException, file: str) -> list[str]:
    shown = [f for f in traceback.extract_tb(error.__traceback__) if os.path.realpath(f.filename) == os.path.realpath(file)]
    return [f"at {rel(f.filename)}:{f.lineno}  {f.line}" for f in shown[-2:]]


def check_one(fd: int, check: Check, shape: Shape, module: types.ModuleType, ns: dict[str, Any]) -> None:
    """Read one value and compare it, in this process: only the verdict, and the two texts of a failure, cross back."""
    try:
        message = evaluate(check.message, shape, module, ns) if check.message is not None else None
        reading = evaluate(check.reading, shape, module, ns)
        expected = evaluate(check.expected, shape, module, ns)
    except Exception as error:
        emit(fd, check=check.label, line=check.line, status="fail", why=f"reading it raised {type(error).__name__}: {error}", trace=frames(error, shape.file))
        return
    why = differs(expected, reading)
    if why is None:
        emit(fd, check=check.label, line=check.line, status="pass", reading=short(reading))
        return
    if not why and short(expected) == short(reading):
        why = f"same text, types {kind(expected)} and {kind(reading)}"
    emit(fd, check=check.label, line=check.line, status="fail", asks=message, expected=f"{short(expected)} ({kind(expected)})", reading=f"{short(reading)} ({kind(reading)})", why=why)


def is_handle(value: object) -> bool:
    if isinstance(value, mmap.mmap):
        return True
    try:
        return isinstance(getattr(value, "fileno")(), int)
    except Exception:
        return False


def in_case(fd: int, case: Case[Any], run: Run) -> None:
    signal.alarm(SECONDS)
    shape = run.shapes[case.test]
    module = module_of(case.test)
    lost: list[str] = []
    sys.unraisablehook = lambda u: lost.append(f"{type(u.exc_value).__name__}: {u.exc_value}")
    threading.excepthook = lambda h: lost.append(f"in a thread, {h.exc_type.__name__}: {h.exc_value}")
    checks = shape.checks
    try:
        try:
            inputs = build(case, run, top=True)
        except Blocked as why:
            for c in checks:
                emit(fd, check=c.label, line=c.line, status="blocked", why=str(why))
            return
        except Exception as error:
            emit(fd, check="inputs", line=0, status="fail", why=f"building an input raised {type(error).__name__}: {error}", trace=frames(error, shape.file))
            return
        for name, value in inputs.items():  # a stand-in matches the type that the test declares
            allowed = annotation_types(shape.params[name].annotation, vars(module))
            if allowed and isinstance(shape.params[name].default, Use) and name not in case.inputs and not fits(value, allowed):
                emit(fd, check="inputs", line=0, status="fail", why=f"the fixture of {name} gives {a(kind(value))}, and the test declares {' | '.join(t.__name__ for t in allowed)}")
                return
        act = compile_act(shape, module)
        SEEN.clear()
        with warnings.catch_warnings(record=True) as seen:
            warnings.simplefilter("always")
            try:
                ns = act(**inputs)
                error = None
            except (Missed, TryNotRaised) as missed:
                ns, error = None, missed
            except Exception as raised:
                ns, error = None, raised
        for text in SEEN:
            emit(fd, check=text.split(" ", 1)[0], line=0, status="pass", reading=text.split(" ", 1)[1])
        if error is not None:
            label = "raises" if isinstance(error, Missed) else "act"
            why = str(error) if isinstance(error, (Missed, TryNotRaised)) else f"the act raised {type(error).__name__}: {error}"
            emit(fd, check=label, line=0, status="fail", why=why, trace=frames(error, shape.file))
            for c in checks:
                emit(fd, check=c.label, line=c.line, status="blocked", why="the act did not end as the test says")
            return
        assert ns is not None
        for w in seen:
            emit(fd, check="warning", line=0, status="fail", why=f"a warning that the test does not expect: {w.category.__name__}: {w.message}", trace=[f"at {rel(w.filename)}:{w.lineno}"])
        handles = {n for n in ns if is_handle(ns[n]) and sum(n in c.names for c in checks) >= 2}
        stock_readers = {id(c) for c in checks if c.names & set(run.stock_inputs)}
        runnable = []
        for c in checks:
            held = sorted(c.names & handles)
            if held:
                emit(fd, check=c.label, line=c.line, status="refused", why=f"{held[0]} is an open file, socket, pipe or mmap, and more than one check reads it: a fork cannot seal it")
                continue
            unsealed = [n for n in c.names & set(run.stock_inputs) if not isinstance(run.stock_inputs[n], Checkpoints) and sum(n in d.names for d in checks) >= 2]
            if unsealed:
                emit(fd, check=c.label, line=c.line, status="refused", why=f"{unsealed[0]} is a stock with no checkpoint, and more than one check reads it")
                continue
            runnable.append(c)
        inside = [c for c in runnable if id(c) not in stock_readers]
        for c in inside[:-1]:  # each check but the last in a fork of its own: none can change what another reads (#964)
            pid = os.fork()
            if pid == 0:
                check_one(fd, c, shape, module, ns)
                os._exit(0)
            os.waitpid(pid, 0)
        for c in (c for c in runnable if id(c) in stock_readers):  # a check of a stock runs in place, between a checkpoint and a restore (#965)
            stock = next(run.stock_inputs[n] for n in c.names if n in run.stock_inputs)
            value = ns[next(n for n in c.names if n in run.stock_inputs)]
            mark = stock.checkpoint(value) if isinstance(stock, Checkpoints) else None
            check_one(fd, c, shape, module, ns)
            if isinstance(stock, Checkpoints):
                stock.restore(value, mark)
        if inside:
            check_one(fd, inside[-1], shape, module, ns)
        for text in lost:
            emit(fd, check="lost error", line=0, status="fail", why=f"an error that no code saw: {text}")
    finally:
        for stock_type, (stock, value, mark) in run.stocks.items():
            try:
                if isinstance(stock, Samples):
                    stock.discard(value)
                else:
                    assert isinstance(stock, Checkpoints)
                    stock.restore(value, mark)
            except Exception as error:
                emit(fd, check="stock", line=0, status="fail", why=f"{stock_type.__name__} was not discarded or restored: {type(error).__name__}: {error}")


def run_case(case: Case[Any], run: Run) -> tuple[list[dict[str, Any]], str]:
    sys.stdout.flush()
    sys.stderr.flush()
    r, w = os.pipe()
    out = tempfile.TemporaryFile()
    pid = os.fork()
    if pid == 0:
        os.close(r)
        os.dup2(out.fileno(), 1)
        os.dup2(out.fileno(), 2)
        try:
            in_case(w, case, run)
        except BaseException as error:  # a fault of the runner, shown as it is
            emit(w, check="runner", line=0, status="fail", why=f"{type(error).__name__}: {error}")
        sys.stdout.flush()
        sys.stderr.flush()
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


# ── ⑤ REPORT ────────────────────────────────────────────────────────────────────────


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
            print(f"   ✓ {label}   {res.get('reading', '')}")
            continue
        if res["status"] == "blocked":
            status = "blocked" if status == "pass" else status
            print(f"   ⊘ {label}   blocked: {res['why']}")
            continue
        status = "fail"
        mark = "!" if res["status"] == "refused" else "✗"
        line = f" · assert at line {res['line']}" if res["line"] else ""
        print(f"   {mark} {label}   {where}{line}")
        if res.get("asks"):
            print(f"       asks      {res['asks']}")
        for key in ("expected", "reading"):
            if key in res:
                print(f"       {key:<9} {res[key]}")
        if res.get("why"):
            print(f"       {'why' if 'expected' in res else 'what':<9} {res['why']}")
        for text in res.get("trace", []):
            print(f"                 {text}")
        if case is not None:
            print(f"       inputs    {show_inputs(case)}")
    if status != "pass" and captured.strip():
        print("       output    " + "\n                 ".join(captured.strip().splitlines()[-8:]))
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


def load(path: Path) -> types.ModuleType:
    """Import a test file under a module name of its own: two files named alike in two folders stay two modules."""
    real, name, n = os.path.realpath(path), path.stem, 1
    while name in sys.modules and os.path.realpath(getattr(sys.modules[name], "__file__", None) or "") != real:
        n += 1
        name = f"{path.stem}_{n}"
    if name in sys.modules:
        return sys.modules[name]
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    try:
        spec.loader.exec_module(module)
    except BaseException:
        del sys.modules[name]
        raise
    return module


def run_file(path: Path, tally: Tally) -> None:
    real = os.path.realpath(path)
    src = Source.read(path)
    sys.path.insert(0, str(path.resolve().parent))
    before = len(CREATED)
    try:
        module = load(path)
    except Exception as error:
        print(f"── {rel(path)}\n   ! the file is refused: importing it raised {type(error).__name__}: {error}\n     so none of its tests runs")
        tally.files += 1
        return
    scope = vars(module)
    tests = sorted(
        (v for k, v in scope.items() if k.startswith("test") and isinstance(v, types.FunctionType) and os.path.realpath(inspect.getfile(v)) == real),
        key=lambda f: f.__code__.co_firstlineno,
    )
    cases = [c for c in CREATED[before:] if os.path.realpath(c.file) == real] or [c for c in CREATED if os.path.realpath(c.file) == real]
    if not tests and not cases:
        return
    print(f"── {rel(path)}")
    for name, lines in src.bindings.items():  # two tests with one name: Python keeps the last
        if name.startswith("test") and len(lines) > 1 and isinstance(scope.get(name), types.FunctionType):
            print(f"   ! {name} is defined at lines {', '.join(map(str, lines))}; Python keeps only the last, so the test at line {lines[0]} would be lost")
            tally.refused += len(lines) - 1
    names = {id(v): k for k, v in scope.items() if isinstance(v, Case)}
    shapes = {t: shape_of(t, src) for t in tests}
    unnamed: dict[int, list[Case[Any]]] = {}
    for c in cases:
        if id(c) not in names:
            unnamed.setdefault(c.line, []).append(c)
        elif c.test not in shapes:
            print(f"   ! {names[id(c)]} at line {c.line} is a case of {name_of(c.test)}, which is no test of this file: a test is a function named `test…`")
            tally.refused += 1
    for line, lost in unnamed.items():
        count = f"{len(lost)} cases" if len(lost) > 1 else "a case"
        why = refuse_unnamed(lost[0], src)
        print(f"   ! {count} of {name_of(lost[0].test)} at line {line} {why.replace('is built', 'are built', 1) if len(lost) > 1 else why}")
        tally.refused += len(lost)
    stopped: set[int] = set()
    for test, shape in shapes.items():
        mine = sorted((c for c in cases if c.test is test and id(c) in names), key=lambda c: c.line)
        print(f" {shape.name}  ({rel(path)}:{shape.node.lineno})")
        if shape.faults:
            print("   ! refused:\n       " + "\n       ".join(shape.faults))
            print(f"     so none of its {max(len(mine), 1)} case(s) runs")
            tally.refused += max(len(mine), 1)
            stopped |= {id(c) for c in mine}
            continue
        needs = [n for n, p in shape.params.items() if p.default is inspect.Parameter.empty]
        if not mine and needs:
            print(f"   ! refused: it takes {', '.join(needs)}, and no case gives them: add `name = case({shape.name}, …)`")
            tally.refused += 1
            continue
        run_cases: list[tuple[str, Case[Any]]] = [(names[id(c)], c) for c in mine] or [(shape.name, Case(test, (), {}, str(path), shape.node.lineno))]
        for name, case in run_cases:
            faults = refuse_case(case, shape, src, scope, shapes, written=bool(mine))
            where = f"{rel(case.file)}:{case.line}"
            if faults:
                print(f"   ! {name} refused   {where}\n       " + "\n       ".join(faults))
                tally.refused += 1
                stopped.add(id(case))
                continue
            gone = next((names.get(id(v.case), "?") for v in case.inputs.values() if isinstance(v, Step) and id(v.case) in stopped), None)
            if gone:
                print(f"   ⊘ {name}  blocked: it takes the product of {gone}, which is refused")
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
    print(f"══ {total} case{'s' * (total != 1)}: {tally.passed} passed · {tally.failed} failed · {tally.refused} refused · {tally.blocked} blocked{files}   ({tally.checks} checks held)")
    return 0 if total and total == tally.passed and not tally.files else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
