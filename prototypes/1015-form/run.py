"""Run the tests in files or folders, and print what each case shows. A throwaway form for a blind
rating. Never merge. FORM.md holds the rules a tester reads.

    python run.py [--seed N] [path ...]     each path a file or a folder; the default is the current
                                            folder; `--seed N` repeats the values of `random`

A test is a function named `test…` in a file named `test_….py`. The runner reads its source,
refuses what breaks the form, and imports the file with each test's last asserts rewritten as
checks, as pytest rewrites an assert. Then it runs each test as written, once for each case, in a
process of its own; the checks read in order, and each later check is also read alone, in a fork
taken before any check read.

A chain runs as a fork tree. A stage's act runs once; its process forks once for the stage's own
checks, and once for each later stage, which takes the product from it. The chain is replayed
instead when, after the act, a thread other than the main one is alive, the chain reads a stock, or
the act left a file, socket or pipe open. A stage that failed, was refused, or was blocked blocks
each later stage, and the blocked line names it.

The watch of the act covers the code of the test module only, and a `@contextmanager` function of
that module that a `with` of the test enters. An assert that runs inside a reading is refused. A
refusal in a helper names the test's line that calls it and the helper's own line.

A test may carry `@before(fn)`, `@skip(reason)` and `@skip_if(condition, reason)`, and no other
decorator: `fn()` runs at the start of each case, after the earlier stages it takes and before its
fixtures; a skipped case never runs. A fixture's parameters are fixtures too,
`use(other)`; each factory runs at most once in a case, and a cycle is refused before the run.

A case reports, and never refuses, each private name its test reads, and each module attribute its
act replaced, as seen at the first check. A case line shows its time when it took 1 s or more.

The exit status is 0 when every case passes or is skipped and one passed, and 1 when a case fails,
is refused or is blocked.
`FORM_PATHS=off` and `FORM_ALONE=off` exist only to measure what the watch and the forks cost.
"""

from __future__ import annotations

import ast
import builtins
import contextlib
import difflib
import dis
import hashlib
import importlib.util
import inspect
import json

# The runner's own references: a test that patches `json` must not reach how the runner writes.
_json_dumps, _json_loads = json.dumps, json.loads
import math
import mmap
import operator
import os
import random
import shutil
import signal
import sys
import sysconfig
import tempfile
import threading
import time
import traceback
import types
import typing
import warnings
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, cast

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import cases as form
from cases import (
    BEFORE,
    CREATED,
    OWNED,
    RAISED,
    SEEN,
    SKIPS,
    Case,
    Checkpoints,
    Fingerprints,
    Missed,
    Samples,
    Step,
    Stock,
    Use,
)  # noqa: E402

SECONDS = 30
LONG = 400
HOOK = "__form_check__"
OUT = os.fdopen(
    os.dup(1), "w", buffering=1
)  # the runner's own stream: a test that replaces sys.stdout cannot mute it


CAPTURE: list[list[str]] = []  # the lines said for the case being reported, for FORM_SUMMARY
SUMMARY: list[dict[str, Any]] = []  # one record for each case, written to FORM_SUMMARY at the end


def say(text: str) -> None:
    OUT.write(text + "\n")
    OUT.flush()
    if CAPTURE:
        CAPTURE[-1].append(text)


def record(file: str, test: str | None, case: str | None, status: str, lines: list[str]) -> None:
    SUMMARY.append(dict(file=file, test=test, case=case, status=status, text="\n".join(lines)))


def flush() -> None:
    """Flush what the test may have left in sys.stdout and sys.stderr, which it may also have
    replaced, or set to None."""
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.flush()
        except Exception:
            pass


# ── ① PARSE: read each test, and refuse before anything runs ──────────

LOGIC: dict[
    type, str
] = {  # the paths of the act (if, loops, try, raise, assert) are seen while the case runs
    ast.With: "a `with` inside a loop or another block: a setup wraps statements of the act itself",
    ast.AsyncWith: "an `async with`",
    ast.Global: "a `global`: take the value as an input",
    ast.Nonlocal: "a `nonlocal`: take the value as an input",
    ast.Yield: "a `yield`: a test returns its product",
    ast.YieldFrom: "a `yield`: a test returns its product",
    ast.Await: "an `await`: this form runs no coroutine",
}
EXPECTERS = {"raises", "warns"}
SKIPPERS = {"skip": 1, "skip_if": 2}  # each decorator that skips a test → where its reason sits
OPS: dict[type, tuple[str, Callable[[Any, Any], bool]]] = {
    ast.Eq: ("==", operator.eq),
    ast.NotEq: ("!=", operator.ne),
    ast.Lt: ("<", operator.lt),
    ast.LtE: ("<=", operator.le),
    ast.Gt: (">", operator.gt),
    ast.GtE: (">=", operator.ge),
    ast.In: ("in", lambda a, b: a in b),
    ast.NotIn: ("not in", lambda a, b: a not in b),
    ast.Is: ("is", operator.is_),
    ast.IsNot: ("is not", operator.is_not),
}


@dataclass
class Check:
    """One of a test's last asserts: one comparison, with the reading on one side and the expected
    value on the other; or `not reading`, `isinstance(reading, T)`, `reading.startswith(x)` or
    `reading.endswith(x)`, whose reading is always on the left."""

    node: ast.Assert
    op: type | None
    reading_left: bool
    names: set[str]  # the names the reading reads
    form: str = "compare"  # compare · not · isinstance · startswith · endswith
    kind_text: str = ""  # for `isinstance`: the class, as written

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
    expects: list[str] = field(default_factory=list)  # `raises` and `warns`, as written
    by_type: list[tuple[int, str, str | None]] = field(
        default_factory=list
    )  # each `raises` or `warns` with no `match=`: its line, its text, and the name `as` binds
    trys: dict[int, tuple[int, str]] = field(
        default_factory=dict
    )  # each `try` of the form's shape: its line → the line of its `except`, and what it expects
    private: list[tuple[int, str]] = field(
        default_factory=list
    )  # each private name the body reads, with its line: reported, never refused
    expected_inputs: set[str] = field(
        default_factory=set
    )  # inputs that a case fills with an expected value
    made_in_raises: set[str] = field(default_factory=set)
    raises_spans: list[tuple[int, int]] = field(
        default_factory=list
    )  # the lines of each `with raises(…):`
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
    values: dict[str, ast.expr] = field(
        default_factory=dict
    )  # each name bound once, at module level, to its value

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
        return next(
            (s for s in self.tree.body if s.lineno <= line <= (s.end_lineno or s.lineno)), None
        )

    def function(self, name: str) -> ast.FunctionDef | ast.AsyncFunctionDef | None:
        defs = [
            s
            for s in self.tree.body
            if isinstance(s, (ast.FunctionDef, ast.AsyncFunctionDef)) and s.name == name
        ]
        return defs[-1] if defs else None  # Python keeps the last `def` of a name


def split(node: ast.FunctionDef) -> tuple[list[ast.stmt], list[ast.Assert], ast.Return | None]:
    """A test's body: the act, its checks, and a last `return`. The act may end inside a `with`,
    whose last asserts are checks too, run before the asserts after it. The act keeps that
    `with`; its checks are listed apart."""
    body = node.body
    if (
        body
        and isinstance(body[0], ast.Expr)
        and isinstance(body[0].value, ast.Constant)
        and isinstance(body[0].value.value, str)
    ):
        body = body[1:]
    ret = body[-1] if body and isinstance(body[-1], ast.Return) else None
    if ret is not None:
        body = body[:-1]
    act, checks = tail(body)
    return act, checks, ret


def is_setup(stmt: ast.stmt) -> bool:
    """A `with` that sets something up for the statements inside it: any `with` but `raises` and
    `warns`."""
    return isinstance(stmt, ast.With) and not any(expecter(item) for item in stmt.items)


def tail(body: list[ast.stmt]) -> tuple[list[ast.stmt], list[ast.Assert]]:
    """A run of statements: its act, and its checks, the asserts at its end, after those at the end
    of its last `with`."""
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
    return (
        f"; did you mean {close[0]!r}?"
        if close
        else f"; it has {', '.join(map(repr, choices)) or 'none'}"
    )


def literal_or_why(node: ast.expr, src: Source) -> str | None:
    if isinstance(node, ast.Constant):
        return None
    if (
        isinstance(node, ast.UnaryOp)
        and isinstance(node.op, (ast.USub, ast.UAdd))
        and isinstance(node.operand, ast.Constant)
    ):
        return None
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        return next((w for w in (literal_or_why(e, src) for e in node.elts) if w), None)
    if isinstance(node, ast.Dict):
        return next(
            (
                w
                for w in (
                    literal_or_why(e, src) for e in [*node.keys, *node.values] if e is not None
                )
                if w
            ),
            None,
        )
    if (
        isinstance(node, ast.Name) and node.id in src.values
    ):  # a name passes when its one binding is a literal
        return literal_or_why(src.values[node.id], src)
    if isinstance(
        node, ast.BinOp
    ):  # operators on literals, such as `"7" * 20 + "\n"`, compute nothing of the code's
        return literal_or_why(node.left, src) or literal_or_why(node.right, src)
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.Invert):
        return literal_or_why(node.operand, src)
    return (
        f"computes `{ast.unparse(node)}`; an expected value is a literal, or operators on "
        "literals, so the test cannot compute what the code computes"
    )


def logic_in(node: ast.AST) -> list[str]:
    """Each statement that this form does not take, read from code. The paths of the act, and a
    function that the test calls or hands to the code, are seen while the case runs, so the
    walk does not enter a nested function or class."""
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
    """An expected value is a literal, an input of the test, or a list, tuple, set or dict
    of these."""
    if isinstance(node, ast.Name) and node.id in shape.params:
        return None
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        return next((w for w in (expected_or_why(e, shape, src) for e in node.elts) if w), None)
    if isinstance(node, ast.Dict):
        return next(
            (
                w
                for w in (
                    expected_or_why(e, shape, src)
                    for e in [*node.keys, *node.values]
                    if e is not None
                )
                if w
            ),
            None,
        )
    return literal_or_why(node, src)


def inputs_named(node: ast.expr, shape: Shape) -> set[str]:
    return {n.id for n in ast.walk(node) if isinstance(n, ast.Name) and n.id in shape.params}


def stored(nodes: list[ast.stmt]) -> set[str]:
    module = ast.Module(body=nodes, type_ignores=[])
    return {
        n.id for n in ast.walk(module) if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Store)
    }


def expecter(item: ast.withitem) -> str | None:
    call = item.context_expr
    if isinstance(call, ast.Call):
        fn = call.func
        name = (
            fn.id
            if isinstance(fn, ast.Name)
            else fn.attr
            if isinstance(fn, ast.Attribute)
            else None
        )
        if name in EXPECTERS:
            return name
    return None


def read_with(
    stmt: ast.With, shape: Shape, src: Source, walk: Callable[[list[ast.stmt]], None]
) -> None:
    if is_setup(
        stmt
    ):  # a setup: what it sets up holds for the statements inside it, and ends with them
        for item in stmt.items:
            shape.faults += logic_in(item.context_expr)
        walk(stmt.body)
        return
    kind = expecter(stmt.items[0]) if len(stmt.items) == 1 else None
    if kind is None:
        shape.faults.append(f"line {stmt.lineno}: `raises` and `warns` each take a `with` of their own")
        return
    if len(stmt.body) != 1:
        shape.faults.append(
            f"line {stmt.lineno}: `with {kind}(…):` holds one statement, the one that raises or "
            f"warns; it holds {len(stmt.body)}"
        )
    if kind == "raises":  # a line of the act that runs after it is seen while the case runs
        shape.raises_spans.append((stmt.lineno, stmt.end_lineno or stmt.lineno))
    call = stmt.items[0].context_expr
    assert isinstance(call, ast.Call)
    for arg in [*call.args[1:], *(k.value for k in call.keywords)]:
        if (
            not (isinstance(arg, ast.Name) and arg.id in shape.params)
            and (why := literal_or_why(arg, src)) is not None
        ):
            shape.faults.append(f"line {stmt.lineno}: `{kind}(…)` {why}")
        shape.expected_inputs |= inputs_named(arg, shape)
    match = call.args[1] if len(call.args) > 1 else next(
        (k.value for k in call.keywords if k.arg == "match"), None
    )
    if match is None or (isinstance(match, ast.Constant) and match.value is None):
        bound = stmt.items[0].optional_vars
        shape.by_type.append(
            (stmt.lineno, ast.unparse(call), bound.id if isinstance(bound, ast.Name) else None)
        )
    shape.expects.append(ast.unparse(call))
    for inner in stmt.body:
        shape.faults += logic_in(inner)
    if kind == "raises":
        shape.made_in_raises |= stored(stmt.body)


AFFIXES = ("startswith", "endswith")


def check_parts(test: ast.expr) -> tuple[str, ast.expr, ast.expr | None] | None:
    """A check's shape, and its two sides: one comparison, `not reading`, `isinstance(reading, T)`,
    `reading.startswith(x)` or `reading.endswith(x)`. None for any other assert."""
    if isinstance(test, ast.Compare) and len(test.ops) == 1 and type(test.ops[0]) in OPS:
        return "compare", test.left, test.comparators[0]
    if isinstance(test, ast.UnaryOp) and isinstance(test.op, ast.Not):
        return "not", test.operand, None
    if (
        isinstance(test, ast.Call)
        and not test.keywords
        and not any(isinstance(arg, ast.Starred) for arg in test.args)
    ):
        fn = test.func
        if isinstance(fn, ast.Name) and fn.id == "isinstance" and len(test.args) == 2:
            return "isinstance", test.args[0], test.args[1]
        if isinstance(fn, ast.Attribute) and fn.attr in AFFIXES and len(test.args) == 1:
            return fn.attr, fn.value, test.args[0]
    return None


def dotted(node: ast.expr) -> list[str] | None:
    """`a.b.C` → ["a", "b", "C"]; None when the node is no name or dotted name."""
    if isinstance(node, ast.Name):
        return [node.id]
    if isinstance(node, ast.Attribute) and (head := dotted(node.value)) is not None:
        return [*head, node.attr]
    return None


def class_or_why(node: ast.expr, scope: dict[str, Any]) -> str | None:
    """The `T` of `isinstance(reading, T)`: a name or a dotted name bound to a class, or a tuple of
    them."""
    parts = node.elts if isinstance(node, ast.Tuple) else [node]
    for part in parts or [node]:
        chain = dotted(part)
        if chain is None:
            return f"`{ast.unparse(part)}` is no name or dotted name"
        value: object = scope.get(chain[0], getattr(builtins, chain[0], None))
        for attr in chain[1:]:
            value = getattr(value, attr, None)
        if not isinstance(value, type):
            return f"`{ast.unparse(part)}` is bound to no class at module level"
    return None


ALLOWED_CHECKS = (
    "a check compares a reading with the value you expect, with one operator: "
    "`assert reading == expected`, `in`, `<`, `is None`; or it is `assert not reading`, "
    "`assert isinstance(reading, T)`, `assert reading.startswith(x)` or "
    "`assert reading.endswith(x)`"
)


def read_check(stmt: ast.Assert, shape: Shape, src: Source) -> Check | None:
    test = stmt.test
    where = f"line {stmt.lineno}"
    parts = check_parts(test)
    if parts is None:
        shape.faults.append(f"{where}: {ALLOWED_CHECKS}")
        return None
    form, left, right = parts
    kind_text = ""
    if form == "compare":
        assert right is not None
        reading_left = (
            expected_or_why(right, shape, src) is None
            or expected_or_why(left, shape, src) is not None
        )
    else:  # the reading is the one value, or the left of the call
        reading_left = True
    reading, expected = (left, right) if reading_left else (right, left)
    assert reading is not None
    if form == "isinstance":
        assert expected is not None
        kind_text = ast.unparse(expected)
        if (why := class_or_why(expected, shape.fn.__globals__)) is not None:
            shape.faults.append(
                f"{where}: `isinstance(reading, T)` takes a class as T: {why}; write the class's "
                "name, a dotted name, or a tuple of them"
            )
    elif expected is not None:
        if (why := expected_or_why(expected, shape, src)) is not None:
            shape.faults.append(
                f"{where}: the expected value {why}; write a literal, or take it as an input of "
                "the case"
            )
        shape.expected_inputs |= inputs_named(expected, shape)
    for fault in logic_in(reading):
        shape.faults.append(
            f"{where}: the reading {fault.split(' holds ', 1)[1]}" if " holds " in fault else fault
        )
    names = {n.id for n in ast.walk(reading) if isinstance(n, ast.Name)}
    if not names:
        shape.faults.append(
            f"{where}: `{ast.unparse(reading)}` reads no value; read what the act made"
        )
    message = stmt.msg
    if message is not None and not (
        isinstance(message, ast.JoinedStr)
        or (isinstance(message, ast.Constant) and isinstance(message.value, str))
    ):
        shape.faults.append(f"{where}: the message of a check is a string, or an f-string")
    late = sorted(names & shape.made_in_raises)
    if late:
        shape.faults.append(
            f"{where} reads {', '.join(late)}, which the act assigns inside "
            "`with raises(…):`, so it never exists; read `caught.value` with "
            "`with raises(…) as caught:`"
        )
    op = type(test.ops[0]) if isinstance(test, ast.Compare) else None
    return Check(stmt, op, reading_left, names, form, kind_text)


def shape_of(fn: types.FunctionType, src: Source) -> Shape:
    node = src.function(fn.__name__)
    if not isinstance(node, ast.FunctionDef):
        stub = ast.parse(f"def {fn.__name__}(): pass").body[0]
        assert isinstance(stub, ast.FunctionDef)
        stub.lineno = node.lineno if node else fn.__code__.co_firstlineno
        shape = Shape(fn, stub, str(src.path))
        shape.faults.append(
            "an async test: this form runs no coroutine"
            if node
            else "its source is not a `def` of this file"
        )
        return shape
    shape = Shape(fn, node, str(src.path))
    for dec in node.decorator_list:
        which = form_decorator(dec, fn.__globals__)
        if which is None:
            shape.faults.append(
                f"line {dec.lineno}: `@{ast.unparse(dec)}`: a test takes no decorator but the "
                "form's `@before(fn)`, `@skip(reason)` and `@skip_if(condition, reason)`; give its "
                f"cases with `name = case({fn.__name__}, …)`"
            )
        elif which in SKIPPERS:
            assert isinstance(dec, ast.Call)
            reason = dec.args[SKIPPERS[which] - 1]
            if not (
                isinstance(reason, ast.Constant)
                and isinstance(reason.value, str)
                and reason.value.strip()
            ):
                shape.faults.append(
                    f"line {dec.lineno}: `@{which}(…)` takes its reason as a string literal that is "
                    f"not empty, and it has `{ast.unparse(reason)}`: say why the test does not run"
                )
    if node.args.vararg or node.args.kwarg:
        shape.faults.append(f"line {node.lineno}: `*args` or `**kwargs`: name each input")
    act, asserts, ret = split(node)
    is_check = {id(a) for a in asserts}

    def walk(stmts: list[ast.stmt]) -> None:
        stmts = [s for s in stmts if id(s) not in is_check]
        for stmt in stmts:
            if isinstance(stmt, ast.Return):
                shape.faults.append(
                    f"line {stmt.lineno} returns before the checks: `return` is the last line of a "
                    "test"
                )
            elif isinstance(stmt, ast.With):
                read_with(stmt, shape, src, walk)
            elif isinstance(stmt, ast.Try) and (expects := try_of_the_form(stmt)) is not None:
                shape.trys[stmt.lineno] = (stmt.handlers[0].lineno, expects)
                for inner in [*stmt.body, *stmt.handlers[0].body]:
                    shape.faults += logic_in(inner)
            else:
                shape.faults += logic_in(stmt)

    walk(act)
    for stmt in asserts:
        if check := read_check(stmt, shape, src):
            shape.checks.append(check)
    for line, text, bound in shape.by_type:  # a raise or a warning checked by its type alone
        if bound is None or not any(
            isinstance(n, ast.Attribute)
            and n.attr == "value"
            and isinstance(n.value, ast.Name)
            and n.value.id == bound
            for c in shape.checks
            for n in ast.walk(c.node.test)
        ):
            shape.faults.append(
                f"line {line}: `with {text}:` checks the type alone; add `match=r\"…\"`, or "
                f"check `caught.value` after `with {text} as caught:`"
            )
    shape.private = private_reads(node)
    shape.returns = ret.value if ret else None
    if shape.returns is not None:
        shape.faults += logic_in(shape.returns)
        if any(e.startswith("raises") for e in shape.expects):
            shape.faults.append(
                f"line {shape.returns.lineno}: a test that expects a raise makes no product, so it "
                "returns nothing"
            )
    # "checks nothing" is seen while the case runs, when no check ran
    for name, p in shape.params.items():
        if isinstance(p.default, Use):
            for fault in fixture_faults(name, p.default.factory, []):
                if fault not in shape.faults:
                    shape.faults.append(fault)
    return shape


FORM_DECORATORS = {"before": 1, "skip": 1, "skip_if": 2}  # each decorator of the form → its arguments


def form_decorator(dec: ast.expr, scope: dict[str, Any]) -> str | None:
    """`@before(fn)`, `@skip(reason)` or `@skip_if(condition, reason)`, where each is the form's
    own: its name, or None."""
    if not (isinstance(dec, ast.Call) and not dec.keywords):
        return None
    func = dec.func
    if isinstance(func, ast.Name):
        value = scope.get(func.id)
    elif isinstance(func, ast.Attribute) and isinstance(func.value, ast.Name):
        value = getattr(scope.get(func.value.id), func.attr, None)
    else:
        return None
    for name, count in FORM_DECORATORS.items():
        if value is getattr(form, name) and len(dec.args) == count:
            return name
    return None


def try_of_the_form(stmt: ast.Try) -> str | None:
    """`try:` with one statement and one `except SomeError:` (a name or a dotted name, `as` allowed),
    and no `else` or `finally`: what it expects. None for any other `try`."""
    if len(stmt.body) != 1 or len(stmt.handlers) != 1 or stmt.orelse or stmt.finalbody:
        return None
    caught = stmt.handlers[0].type
    return ast.unparse(caught) if caught is not None and dotted(caught) is not None else None


TRY_SHAPE = (
    "a `try` of the form holds one statement and one `except SomeError:`, with no `else` and no "
    "`finally`, and fails the case when the statement does not raise SomeError"
)


def is_private(name: str) -> bool:
    return name.startswith("_") and name != "_" and not (name.startswith("__") and name.endswith("__"))


def private_reads(node: ast.FunctionDef) -> list[tuple[int, str]]:
    """Each private name or attribute that the body of a test reads, with its nested defs, lambdas
    and classes: once a name on a line. A name the test binds itself (an input, a local) is its
    own."""
    body = ast.Module(body=node.body, type_ignores=[])
    own = {a.arg for a in ast.walk(node.args) if isinstance(a, ast.arg)}
    for n in ast.walk(body):
        if isinstance(n, ast.Name) and not isinstance(n.ctx, ast.Load):
            own.add(n.id)
        elif isinstance(n, ast.arg):
            own.add(n.arg)
        elif isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            own.add(n.name)
        elif isinstance(n, ast.ExceptHandler) and n.name:
            own.add(n.name)
        elif isinstance(n, ast.alias):
            own.add((n.asname or n.name).split(".")[0])
    found: dict[tuple[int, str], None] = {}
    for n in ast.walk(body):
        if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Load) and is_private(n.id):
            if n.id not in own:
                found[(n.lineno, n.id)] = None
        elif isinstance(n, ast.Attribute) and is_private(n.attr):
            found[(n.lineno, n.attr)] = None
    return sorted(found)


def fixture_params(factory: object) -> list[inspect.Parameter]:
    try:
        params = inspect.signature(factory).parameters.values()  # type: ignore[arg-type]
    except (TypeError, ValueError):  # a builtin with no signature takes nothing
        return []
    return [p for p in params if p.kind not in (p.VAR_POSITIONAL, p.VAR_KEYWORD)]


def fixture_faults(name: str, factory: object, trail: list[object]) -> list[str]:
    """What refuses the fixture of one input, through each fixture it uses: a stock with no watch, a
    cycle, and a parameter that is no fixture."""
    if isinstance(factory, Case):
        return []
    through = f" (through {' → '.join(name_of(f) for f in trail)})" if trail else ""
    if isinstance(factory, type) and issubclass(factory, Stock):
        stock, faults = factory, []
        if inspect.isabstract(stock):
            missing = ", ".join(sorted(getattr(stock, "__abstractmethods__", ())))
            return [
                f"the input {name}{through}: the stock {stock.__name__} lacks {missing}; a "
                "misspelled method leaves it abstract"
            ]
        if not issubclass(stock, (Samples, Checkpoints)):
            faults.append(
                f"the input {name}{through}: the stock {stock.__name__} offers neither Samples nor "
                "Checkpoints, so nothing seals it for the case"
            )
        if not issubclass(stock, (Fingerprints, Checkpoints)):  # by its type, not the names read
            faults.append(
                f"the input {name}{through}: the stock {stock.__name__} offers neither "
                "Fingerprints nor Checkpoints, so a check that changes it cannot be seen"
            )
        return faults
    if factory in trail:
        loop = trail[trail.index(factory) :] + [factory]
        return [
            f"the input {name}: the fixtures {' → '.join(name_of(f) for f in loop)} form a cycle, "
            "so none of them can run first"
        ]
    faults: list[str] = []
    for p in fixture_params(factory):
        if isinstance(p.default, Use):
            faults += fixture_faults(name, p.default.factory, [*trail, factory])
        elif p.default is p.empty or inspect.isfunction(factory):
            # a class keeps its own defaults; a factory function takes only fixtures
            faults.append(
                f"the input {name}{through}: the fixture {name_of(factory)} takes {p.name}, whose "
                f"default is no `use(…)`; a fixture takes only fixtures: `{p.name}: T = use(…)`"
            )
    return faults


IMMUTABLE = (str, bytes, int, float, bool, type(None), tuple)


def annotation_types(annotation: Any, scope: dict[str, Any]) -> tuple[type, ...] | None:
    """The run-time types an annotation allows, or None when it is not read (a name only a stub
    declares, Any, object)."""
    if annotation is inspect.Parameter.empty:
        return None
    if isinstance(annotation, str):
        try:
            annotation = eval(
                annotation, scope
            )  # a name that exists only for the type checker stays unread
        except Exception:
            return None
    if annotation is None:
        return (type(None),)
    if (
        isinstance(annotation, types.UnionType)
        or getattr(annotation, "__origin__", None) is typing.Union
    ):
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
        if (
            isinstance(node, ast.Call)
            and node.args
            and isinstance(node.args[0], ast.Name)
            and node.args[0].id == name_of(case.test)
        ):
            return node
    return None


def refuse_case(
    case: Case[Any],
    shape: Shape,
    src: Source,
    scope: dict[str, Any],
    shapes: dict[Any, Shape],
    written: bool = True,
) -> list[str]:
    """Every fault of one case, all at once."""
    faults: list[str] = []
    params = shape.params
    if case.args:
        faults.append(
            f"passes {len(case.args)} input(s) by position; name each input, as "
            f"`case({shape.name}, name=…)`"
        )
    for key in case.inputs:
        if key not in params:
            faults.append(
                f"gives {key!r}, which is no input of {shape.name}{hint(key, list(params))}"
            )
    for name, p in params.items():
        if name not in case.inputs and p.default is inspect.Parameter.empty:
            faults.append(f"gives no {name!r}, an input of {shape.name} with no default")
    call = case_call(case, src)
    if call is None and written:
        faults.append(
            f"is not written `case({shape.name}, …)`; call `case` with the test's own name, "
            "so the runner can read the expected values"
        )
    elif call is not None and any(kw.arg is None for kw in call.keywords):
        faults.append("passes inputs with `**`; name each input, as `case(…, name=…)`")
    for kw in call.keywords if call else []:
        if kw.arg in shape.expected_inputs and (why := literal_or_why(kw.value, src)) is not None:
            faults.append(f"gives {kw.arg}=…, an expected value, which {why}")
    for key, value in case.inputs.items():
        if key in params and not isinstance(value, (Step, Use)):
            allowed = annotation_types(params[key].annotation, scope)
            if allowed and not fits(value, allowed):
                faults.append(
                    f"gives {key}={short(value)} ({kind(value)}), and {shape.name} declares "
                    f"{' | '.join(t.__name__ for t in allowed)}"
                )
        if isinstance(value, Use):  # a fixture that the case line gives
            faults += fixture_faults(key, value.factory, [])
        if isinstance(value, Step):
            earlier = shapes.get(value.case.test)
            if earlier and earlier.returns is None:
                faults.append(
                    f"takes {key} from the product of the case at line {value.case.line}, and "
                    f"{earlier.name} returns nothing"
                )
    for key, p in params.items():  # use(stage), given in the case or as the default
        given = case.inputs.get(key, p.default)
        if isinstance(given, Use) and isinstance(given.factory, Case):
            earlier = shapes.get(given.factory.test)
            if earlier and earlier.returns is None:
                faults.append(
                    f"takes {key} from use(<the case at line {given.factory.line}>), and "
                    f"{earlier.name} returns nothing"
                )
    for (
        check
    ) in shape.checks:  # a check reads the product, or an input that the act can change
        read = {
            n for n in check.names if n in params or not hasattr(builtins, n)
        }  # `len(x)` reads x
        inputs = read & set(params)
        if (
            inputs
            and not (read - inputs)
            and all(isinstance(case.inputs.get(n, params[n].default), IMMUTABLE) for n in inputs)
        ):
            faults.append(
                f"line {check.line} reads only {', '.join(sorted(inputs))}, a value the act cannot "
                "change; read what the act made"
            )
    return faults


# ── ② IMPORT: each test's last asserts become checks, as pytest rewrites an assert ──


@dataclass
class Setup:
    """A `with` of the act, as the rewrite marked it: where it is, what it sets up, and the line of
    each statement inside."""

    file: str
    line: int
    text: str
    lines: list[int]


SETUPS: list[Setup] = []
MARK, LEFT, STEP, NORAISE = "__form_mark__", "__form_left__", "__form_step__", "__form_noraise__"


def call_stmt(name: str, args: list[object], where: ast.stmt) -> ast.stmt:
    return ast.copy_location(
        ast.Expr(ast.Call(ast.Name(name, ast.Load()), [ast.Constant(a) for a in args], [])), where
    )


class Rewrite(ast.NodeTransformer):
    """`assert a == b, "m"` among a test's checks → `__form_check__("test", 0, lambda: (a, b),
    lambda: "m")`.

    Each setup's body gets a mark after each statement, and a call after the `with`: if the `with`
    let the test go on without its last statements, a raise inside it was swallowed. A mark
    adds no frame to the `with`'s exit.
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
                    assert isinstance(stmt, ast.Assert)
                    test = stmt.test
                    parts = check_parts(test)
                    pair = (
                        ast.Tuple([parts[1], parts[2] or ast.Constant(None)], ast.Load())
                        if parts is not None
                        else test
                    )
                    message = stmt.msg if stmt.msg is not None else ast.Constant(None)
                    call = ast.Call(
                        ast.Name(HOOK, ast.Load()),
                        [
                            ast.Constant(node.name),
                            ast.Constant(place[id(stmt)]),
                            lam(pair),
                            lam(message),
                        ],
                        [],
                    )
                    out.append(ast.copy_location(ast.Expr(call), stmt))
                elif is_setup(stmt):
                    assert isinstance(stmt, ast.With)
                    sid = len(SETUPS)
                    SETUPS.append(
                        Setup(
                            self.file,
                            stmt.lineno,
                            ", ".join(ast.unparse(i) for i in stmt.items),
                            [s.lineno for s in stmt.body],
                        )
                    )
                    inner = fix(stmt.body)
                    marked: list[ast.stmt] = [call_stmt(MARK, [sid, -1], stmt)]
                    for k, s in enumerate(inner):
                        marked += [s, call_stmt(MARK, [sid, k], s)]
                    stmt.body = marked
                    out += [stmt, call_stmt(LEFT, [sid], stmt)]
                elif isinstance(stmt, ast.Try) and (expects := try_of_the_form(stmt)) is not None:
                    # the one statement did not raise: the `else` says so, and fails the case
                    stmt.orelse = [call_stmt(NORAISE, [stmt.lineno, expects], stmt.body[0])]
                    out.append(stmt)
                    if top:
                        out.append(call_stmt(STEP, [], stmt))
                else:
                    out.append(stmt)
                    if top and not isinstance(stmt, ast.Return):
                        out.append(call_stmt(STEP, [], stmt))
            return out

        node.body = fix(node.body, top=True)
        return node


def lam(body: ast.expr) -> ast.Lambda:
    return ast.Lambda(ast.arguments([], [], None, [], [], None, []), body)


FAR: dict[tuple[str, str], Shape] = {}


def far_shape(test: str, file: str) -> Shape:
    """The shape of a test of another file: a stage that this file imports."""
    key = (test, os.path.realpath(file))
    if key not in FAR:
        module = next(
            m
            for m in list(sys.modules.values())
            if os.path.realpath(getattr(m, "__file__", None) or "") == key[1]
        )
        FAR[key] = shape_of(vars(module)[test], Source.read(Path(file)))
    return FAR[key]


def load(path: Path) -> types.ModuleType:
    """Import a test file, rewritten, under a module name of its own: two files named alike stay two
    modules."""
    real, name, n = os.path.realpath(path), path.stem, 1
    while (
        name in sys.modules
        and os.path.realpath(getattr(sys.modules[name], "__file__", None) or "") != real
    ):
        n += 1
        name = f"{path.stem}_{n}"
    if name in sys.modules:
        return sys.modules[name]
    tree = ast.parse(path.read_text(), str(path))
    for stmt in (
        tree.body
    ):  # a test file that this one imports, for a stage, is loaded rewritten first
        names = (
            [stmt.module]
            if isinstance(stmt, ast.ImportFrom) and stmt.module and not stmt.level
            else [a.name for a in stmt.names]
            if isinstance(stmt, ast.Import)
            else []
        )
        for other in names:
            sibling = path.parent / f"{other}.py"
            if other.startswith("test_") and sibling.is_file() and other not in sys.modules:
                load(sibling)
    tree.body = [
        Rewrite(str(path)).visit(s)
        if isinstance(s, ast.FunctionDef) and s.name.startswith("test")
        else s
        for s in tree.body
    ]
    module = types.ModuleType(name)
    module.__file__ = str(path)
    vars(module)[HOOK] = hook
    vars(module)[MARK] = mark
    vars(module)[LEFT] = left
    vars(module)[STEP] = step
    vars(module)[NORAISE] = noraise
    sys.modules[name] = module
    try:
        exec(compile(ast.fix_missing_locations(tree), str(path), "exec"), vars(module))
    except BaseException:
        del sys.modules[name]
        raise
    return module


# ── ③ RUN: a process for each case; each later check read alone, then all in order ──


class Blocked(BaseException):
    pass


@dataclass
class Run:
    """What one case's process holds: the products of replayed steps, each stock it drew, and its
    checks so far."""

    shapes: dict[Any, Shape]
    names: dict[int, str]
    fd: int = -1
    memo: dict[int, object] = field(default_factory=dict)
    stocks: dict[type, tuple[Stock[Any], object, object]] = field(
        default_factory=dict
    )  # stock type → (stock, value, checkpoint)
    stock_inputs: dict[str, Stock[Any]] = field(default_factory=dict)
    replaying: list[str] = field(
        default_factory=list
    )  # the label of each step being replayed, innermost last
    done: set[int] = field(default_factory=set)  # the checks of this case that ran
    reached: dict[int, int] = field(
        default_factory=dict
    )  # each open setup → the last statement inside it that ended
    guard: Guard | None = None
    only: int | None = None  # in a fork that reads one check alone: that check
    alone_fd: int = -1
    forked: bool = False  # the forks that read each later check alone are done
    alone: dict[int, dict[str, Any]] = field(
        default_factory=dict
    )  # each later check → what it read alone
    flagged: bool = (
        False  # the act already failed for its setup or a hook: its paths are not judged
    )
    case: Case[Any] | None = None  # the case this process runs
    tree: dict[int, list[Case[Any]]] = field(
        default_factory=dict
    )  # each stage → the later stages that fork from it
    kids: list[Case[Any]] = field(
        default_factory=list
    )  # this case's later stages, while it may still fork for them
    split: bool = False  # the stage forked for its own checks, or fell back
    cont: bool = False  # this process is the stage's continuation: it reads checks as a replay
    inherited: bool = False  # forked from an earlier stage's continuation
    swallowed: str | None = None  # a setup of the act swallowed a raise: a replay would block
    fds: dict[str, str] = field(
        default_factory=dict
    )  # each open descriptor when the case started → what it points to
    before_act: dict[tuple[str, str], object] | None = None  # each module attribute, as the act began


# ── a setup: what it swallowed, and what its undo raised ──────────


def mark(sid: int, k: int) -> None:
    if RUN:
        RUN[-1].reached[sid] = k
        verify()


def step() -> None:
    """After each statement of the act at the test's own level: is every hook the runner's own
    again?"""
    if RUN:
        verify()


def noraise(line: int, expects: str) -> None:
    """The one statement of a `try` of the form ended, and raised nothing: the case fails."""
    raise Missed("try", f"the `try` at line {line} expects {expects}, and the statement did not raise it")


def verify() -> None:
    guard = RUN[-1].guard if RUN else None
    if guard is not None:
        guard.verify()


def left(sid: int) -> None:
    """The test went on after a setup. If a statement inside it raised, the setup swallowed that
    raise."""
    if not RUN:
        return
    run, setup = RUN[-1], SETUPS[sid]
    verify()
    k = run.reached.pop(sid, None)
    if (
        k is None or k >= len(setup.lines) - 2
    ):  # all ran, or the last statement raised: nothing was skipped
        return
    raised, skipped = setup.lines[k + 1], setup.lines[k + 2 :]
    why = (
        f"`with {setup.text}` swallowed a raise at line {raised}, so "
        f"line{'s' * (len(skipped) > 1)} {', '.join(map(str, skipped))} never ran"
    )
    if run.replaying:
        raise Blocked(f"{run.replaying[-1]}: {why}")
    run.flagged = True
    run.swallowed = why
    emit(
        run.fd,
        check="setup",
        line=0,
        status="fail",
        why=why,
        trace=[f"at {rel(setup.file)}:{setup.line}"],
    )


def setup_at(error: BaseException, file: str, test: str, run: Run) -> tuple[str, str] | None:
    """When the error came from a setup's own `with` line, not from a statement inside it: its
    entry, or its undo."""
    shown = [
        f
        for f in traceback.extract_tb(error.__traceback__)
        if os.path.realpath(f.filename) == os.path.realpath(file) and f.name == test
    ]
    if not shown:
        return None
    line = shown[-1].lineno
    for sid, setup in enumerate(SETUPS):
        if os.path.realpath(setup.file) == os.path.realpath(file) and setup.line <= (line or 0) < (
            setup.lines[0] if setup.lines else setup.line + 1
        ):
            reached = run.reached.get(sid)
            if reached is None:
                return (
                    "setup",
                    f"setting up `with {setup.text}` at line {setup.line} raised "
                    f"{type(error).__name__}: {error}",
                )
            if reached == len(setup.lines) - 1:
                return (
                    "undo",
                    f"the undo of `with {setup.text}` at line {setup.line} raised "
                    f"{type(error).__name__}: {error}",
                )
    return None


# ── the guard: the act may not replace a hook that the runner sees a fault through ─

GUARDED: dict[str, tuple[str, ...]] = {
    "threading": ("excepthook",),
    "sys": ("unraisablehook",),
    "warnings": ("filters", "showwarning", "_showwarnmsg"),
}
SEES = {
    "threading.excepthook": "an error that no code sees in a thread",
    "sys.unraisablehook": "an error that no code can catch, such as one in `__del__`",
    "warnings": "a warning that the test does not expect",
    "signal": "a case that runs too long",
}
MINE = {os.path.realpath(__file__), os.path.realpath(HERE / "cases.py")}
STDLIB = os.path.realpath(sysconfig.get_paths()["stdlib"])


GUARD_MODE = os.environ.get(
    "FORM_GUARD", "line"
)  # line (agreed): fail when a line of the test runs under it · swap: fail at the swap


class Guard:
    """`swap` fails the case the moment the act replaces a hook. `line` only notes where, and fails
    the case when a hook is still not the runner's own after a statement of the test: a window that
    a library opens and closes inside one call of the test passes (freezegun's `catch_warnings`
    inside `freeze_time`)."""

    def __init__(self, run: Run) -> None:
        self.run = run
        self.on = False
        self.tripped: set[str] = set()
        self.pending: dict[
            str, tuple[str, str]
        ] = {}  # family → (name, where), at the swap (line mode)
        self.filters: list[Any] = []
        self.own: dict[str, object] = {}

    def start(self) -> None:
        self.filters = list(warnings.filters)
        self.own = {
            "threading": threading.excepthook,
            "sys": sys.unraisablehook,
            "warnings": warnings.showwarning,
            "signal": signal.getsignal(signal.SIGALRM),
        }
        self.on = True

    def trip(self, name: str) -> None:
        family = name.split(".")[0]
        if not self.on or OWNED[0] or self.run.replaying or family in self.tripped:
            return
        where = next(
            (
                f
                for f in reversed(traceback.extract_stack()[:-1])
                if os.path.realpath(f.filename) not in MINE
                and not os.path.realpath(f.filename).startswith(STDLIB + os.sep)
            ),
            None,
        )
        at = f"at {rel(where.filename)}:{where.lineno}  {where.line}" if where else ""
        if GUARD_MODE == "line":
            self.pending.setdefault(family, (name, at))
            return
        self.fail(family, name, at)

    def fail(self, family: str, name: str, at: str) -> None:
        self.tripped.add(family)
        self.run.flagged = True
        sees = SEES.get(name) or SEES[family]
        emit(
            self.run.fd,
            check="hook",
            line=0,
            status="fail",
            why=f"the act replaced {name}; the runner sees {sees} through it, so while it is "
            "replaced the case cannot show one",
            trace=[at] if at else [],
        )

    def changed(self) -> list[str]:
        """The families whose hook is not the runner's own right now."""
        out = []
        if threading.excepthook is not self.own["threading"]:
            out.append("threading")
        if sys.unraisablehook is not self.own["sys"]:
            out.append("sys")
        if (
            list(warnings.filters) != self.filters
            or warnings.showwarning is not self.own["warnings"]
        ):
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
                def __setattr__(
                    self,
                    key: str,
                    value: object,
                    _attrs: tuple[str, ...] = attrs,
                    _name: str = name,
                ) -> None:
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

        signal.alarm = alarm
        signal.signal = on_signal  # type: ignore[assignment]
        signal.setitimer = timer
        for fn in (
            "simplefilter",
            "filterwarnings",
            "resetwarnings",
        ):  # they change the filters in place: trip where they are called
            real = getattr(warnings, fn)

            def changed(*args: Any, _real: Any = real, _fn: str = fn, **kwargs: Any) -> Any:
                self.trip(f"warnings.filters (by warnings.{_fn})")
                return _real(*args, **kwargs)

            setattr(warnings, fn, changed)


# ── the paths of the act, seen while the case runs ────────────

MON = sys.monitoring
TOOL = 3  # no tool of Python's own: a debugger or coverage of the code under test keeps its id
BODY_ONCE = "the act has one path, so each line of it runs in every case"
BOTH_WAYS = (
    "the act has one path, so each `if`, `x if c else y`, `and`, `or`, `while` and `except` in it "
    "takes both ways in a case, or is not there"
)


CONTEXTLIB = os.path.realpath(contextlib.__file__)


class Paths:
    """Which lines and branches of the test module run, in the case's process. The watch covers the
    code of the test module only: the code under test, and a helper of any other file, are not
    watched, wherever they sit. Code of the test module is part of the act when the test calls it,
    or something the test calls calls it: the test, a helper, a class it builds, a reading of a
    check, and a `@contextmanager` function that a `with` of the test enters (contextlib calls it,
    for the `with`). Code of the module that the code under test calls, such as a callback, is a
    value: its paths are free, but an assert in it is not.

    Each location is switched off once it has told what it can: a line at its first run, a branch
    once it went both ways, a loop once it ran a round. So a loop of a million rounds costs about
    what it costs with no watch."""

    def __init__(self, run: Run, shape: Shape) -> None:
        self.run, self.shape = run, shape
        self.module = os.path.realpath(shape.file)
        self.ours: dict[types.CodeType, bool] = {}
        self.act: set[types.CodeType] = set()
        self.readings: set[types.CodeType] = set()  # the lambdas of each check's reading
        self.called_at: dict[types.CodeType, int] = {}  # a helper → the test's line that reached it
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
        """The lines of the act that start a statement after a `with raises(…):`: none of them may
        run once it caught."""
        if not self.shape.raises_spans:
            return set()
        end = max(e for _, e in self.shape.raises_spans)
        act, _, _ = split(self.shape.node)
        out: set[int] = set()

        def walk(stmts: list[ast.stmt]) -> None:
            for s in stmts:
                if isinstance(s, ast.With):
                    walk(s.body)
                elif (
                    s.lineno > end and s.lineno not in self.checks and not isinstance(s, ast.Return)
                ):
                    out.add(s.lineno)

        walk(act)
        return out

    def is_ours(self, code: types.CodeType) -> bool:
        if (known := self.ours.get(code)) is None:
            known = self.ours[code] = (
                os.path.realpath(code.co_filename) == self.module and code.co_name != "<module>"
            )
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
        """Each branch of the code: a loop's head (with where its body starts), the test of an
        `except`, the exit of a `with`. On 3.12 an exhausted `FOR_ITER` lands past its `END_FOR`,
        not on its argument, so a round is told by the body."""
        if code not in self.kinds:
            found: dict[int, tuple[str, int]] = {}
            prev = None
            listed = list(dis.get_instructions(code))
            for ins, nxt in zip(listed, [*listed[1:], None]):
                if ins.opname == "FOR_ITER" and nxt is not None:
                    found[ins.offset] = ("loop", nxt.offset)
                elif ins.opname.startswith("POP_JUMP_IF") and prev is not None:
                    found[ins.offset] = (
                        ("except", 0)
                        if prev.opname == "CHECK_EXC_MATCH"
                        else ("with", 0)
                        if prev.opname == "WITH_EXCEPT_START"
                        else ("if", 0)
                    )
                prev = ins
            self.kinds[code] = found
        return self.kinds[code]

    def test_line(self, frame: types.FrameType | None) -> int | None:
        """The line of the test that, through the frames above `frame`, runs it: a statement of the
        test, or the line of a check whose reading runs it."""
        while frame is not None:
            if frame.f_code is self.test:
                return frame.f_lineno
            if frame.f_code in self.readings:
                return frame.f_code.co_firstlineno
            frame = frame.f_back
        return None

    def reading_of(self, frame: types.FrameType | None) -> int | None:
        """The line of the check whose reading runs `frame`, if a reading runs it."""
        while frame is not None and frame.f_code is not self.test:
            if frame.f_code in self.readings:
                return frame.f_code.co_firstlineno
            frame = frame.f_back
        return None

    def who(self, code: types.CodeType, called: int | None = None) -> str:
        """For a helper: its name, and the test's line that reached it."""
        if code is self.test:
            return ""
        called = called if called is not None else self.called_at.get(code)
        via = f", reached from line {called} of the test" if called is not None else ""
        return f" ({code.co_name}{via})"

    def where(self, code: types.CodeType, line: int, called: int | None = None) -> str:
        return f"{rel(code.co_filename)}:{line}{self.who(code, called)}"

    def frame_of(self, code: types.CodeType) -> types.FrameType | None:
        frame: types.FrameType | None = sys._getframe(1)
        while frame is not None and frame.f_code is not code:
            frame = frame.f_back
        return frame

    def py_start(self, code: types.CodeType, _offset: int) -> object:
        if not self.on:
            return MON.DISABLE
        if not self.is_ours(code):
            return MON.DISABLE
        if self.run.replaying:
            return None
        start = self.frame_of(code)
        frame = start.f_back if start is not None else None
        caller = frame.f_code if frame is not None else None
        if (
            frame is not None
            and caller is not None
            and os.path.realpath(caller.co_filename) == CONTEXTLIB
            and code.co_flags & inspect.CO_GENERATOR
        ):  # a `@contextmanager` function, which contextlib starts for the `with` that entered it
            frame = frame.f_back
            caller = frame.f_code if frame is not None else None
        if code is self.test or caller in self.act:
            self.act.add(code)
        elif caller is not None and os.path.realpath(caller.co_filename) == os.path.realpath(
            __file__
        ):
            if (
                code.co_name == "<lambda>"
                and code.co_firstlineno in self.checks
                and code.co_filename == self.test.co_filename
            ):
                self.act.add(code)  # a check's reading, which the runner calls
                self.readings.add(code)
        if code in self.act and code is not self.test and code not in self.called_at:
            if (line := self.test_line(start)) is not None:
                self.called_at[code] = line
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
            frame = self.frame_of(code)
            at = self.where(code, line, self.test_line(frame))
            reading = self.reading_of(frame)
            if reading is not None:
                self.faults.append(
                    f"an assert ran inside the reading of the check at line {reading}, at {at}: a "
                    "check is the one comparison; let the helper return what it reads, and compare "
                    "that in the check"
                )
            else:
                self.faults.append(
                    f"an assert ran in the act at {at}: the act ends at the first check, and a "
                    "function handed to the code may swallow an assert; keep what it sees, and "
                    "check that at the end"
                )
        if code is self.test and RAISED[0] and line in self.after_raise:
            self.faults.append(
                f"line {line} ran after the expected raise: `with raises(…):` ends the act, "
                "since after a raise the act has nothing left to run"
            )
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
        line_of = lambda code, src: next(
            (ln for s, e, ln in code.co_lines() if s <= src < e and ln), code.co_firstlineno
        )  # noqa: E731
        for code in sorted(self.act, key=lambda c: (c.co_filename, c.co_firstlineno)):
            if code in self.readings:
                continue  # a reading is one expression; its paths are the check's
            body = {
                ln for _, _, ln in code.co_lines() if ln is not None and ln != code.co_firstlineno
            }
            missed = sorted(body - self.lines.get(code, set()))
            if missed:
                out.append(
                    f"line{'s' * (len(missed) > 1)} {', '.join(map(str, missed))} of "
                    f"{rel(code.co_filename)}{self.who(code)} never ran: {BODY_ONCE}"
                )
            for (c, src), ways in self.ways.items():
                if (
                    c is code
                    and len(ways) == 1
                    and line_of(code, src) not in self.assert_lines(code.co_filename)
                ):  # an assert is told once, above
                    what = self.kind_of(code).get(src, ("if", 0))[0]
                    if (
                        what == "except"
                        and code is self.test
                        and line_of(code, src) in {e for e, _ in self.shape.trys.values()}
                    ):
                        continue  # a `try` of the form: its one statement raised, as it must
                    label = "the `except`" if what == "except" else "the branch"
                    out.append(
                        f"{label} at {self.where(code, line_of(code, src))} went one way: "
                        f"{BOTH_WAYS}" + (f"; {TRY_SHAPE}" if what == "except" else "")
                    )
            for c, src in sorted(self.loops - self.entered, key=lambda x: x[1]):
                if c is code:
                    out.append(
                        f"the loop at {self.where(code, line_of(code, src))} ran no round: "
                        f"{BODY_ONCE}"
                    )
        return out


WATCH: list[Paths] = []  # the watch of this case, set in the case's process
ARMED: set[int] = set()


def arm(files: set[str]) -> None:
    """Set the watch's events on the code of each test module, once, in the runner's own process,
    before any case forks. Each change of events makes every code object of a process instrument
    itself again at its next run, which cost about 10 ms a case on loguru's tests when each case's
    process set them. A case's process sets nothing: it inherits the events, and only points them at
    its own watch."""
    if not ARMED:
        MON.use_tool_id(TOOL, "purba-form")
        # no watch: the runner's own process
        on = lambda name: lambda *a: getattr(WATCH[-1], name)(*a) if WATCH else None  # noqa: E731
        MON.register_callback(TOOL, MON.events.PY_START, on("py_start"))
        MON.register_callback(TOOL, MON.events.LINE, on("line"))
        MON.register_callback(TOOL, MON.events.BRANCH, on("branch"))
    found: dict[int, types.CodeType] = {}

    def add(code: types.CodeType) -> None:
        path = os.path.realpath(code.co_filename)
        if id(code) not in found and path in files and path not in MINE:
            found[id(code)] = code
            for const in code.co_consts:
                if isinstance(const, types.CodeType):
                    add(const)

    def scan(value: object, depth: int = 0) -> None:
        fn = getattr(
            getattr(value, "__func__", value), "__wrapped__", getattr(value, "__func__", value)
        )
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
        if path and os.path.realpath(path) in files and os.path.realpath(path) not in MINE:
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
        return (
            f"<lambda at line {value.__code__.co_firstlineno}>"
            if value.__name__ == "<lambda>"
            else value.__name__
        )
    text = repr(value)
    if len(text) <= LONG:
        return text
    # the end survives: a traceback ends with its point. Each cut falls at a space, so no word is
    # shown in part.
    head, tail = text[: LONG // 2], text[-LONG // 2 :]
    head = head[: head.rfind(" ")] if " " in head else head
    tail = tail[tail.find(" ") + 1 :] if " " in tail else tail
    return head + " … " + tail


def kind(value: object) -> str:
    t = type(value)
    if t is bool:
        return "bool"
    base = next(
        (
            b
            for b in (str, int, float, bytes, list, tuple, dict, set)
            if issubclass(t, b) and t is not b
        ),
        None,
    )
    return f"{t.__name__}, a subclass of {base.__name__}" if base else t.__name__


def a(text: str) -> str:
    return f"{'an' if text[0] in 'aeiou' else 'a'} {text}"


def differs(expected: object, reading: object, at: str = "") -> str | None:
    """None when the reading equals the expected value; else why not ("" when only the values
    differ)."""
    if type(expected) is bool or type(reading) is bool:
        if type(expected) is not type(reading):
            return f"{at}{a(kind(reading))} is not {a(kind(expected))}"
        return None if expected == reading else ""
    if (
        isinstance(expected, float)
        and isinstance(reading, float)
        and math.isnan(expected)
        and math.isnan(reading)
    ):
        return None
    if not isinstance(reading, type(expected)):
        return f"{at}{a(kind(reading))} is not {a(kind(expected))}"
    if isinstance(expected, (list, tuple)):
        assert isinstance(reading, (list, tuple))
        if len(reading) != len(expected):
            return f"{at}{len(reading)} items, not {len(expected)}"
        return next(
            (
                d
                for i, (e, r) in enumerate(zip(expected, reading))
                if (d := differs(e, r, f"{at}[{i}]: ")) is not None
            ),
            None,
        )
    if isinstance(expected, dict):
        assert isinstance(reading, dict)
        if set(reading) != set(expected):
            return f"{at}keys {sorted(map(repr, reading))}, not {sorted(map(repr, expected))}"
        return next(
            (
                d
                for k in expected
                if (d := differs(expected[k], reading[k], f"{at}[{k!r}]: ")) is not None
            ),
            None,
        )
    if reading == expected:
        return None
    if isinstance(expected, str) and isinstance(reading, str):
        at_ = next(
            (i for i, (x, y) in enumerate(zip(expected, reading)) if x != y),
            min(len(expected), len(reading)),
        )
        return (
            f"{at}they differ from character {at_}: expected {expected[at_ : at_ + 20]!r}, read "
            f"{reading[at_ : at_ + 20]!r}"
        )
    return ""


def verdict(check: Check, left: object, right: object) -> str | None:
    """None when the check holds; else why not. `==` and `!=` compare by type, then by value."""
    if check.form == "not":
        return None if not left else ""
    if check.form == "isinstance":
        return None if isinstance(left, cast(type, right)) else ""
    if check.form in AFFIXES:
        return None if getattr(left, check.form)(right) else ""
    reading, expected = (left, right) if check.reading_left else (right, left)
    if check.op is ast.Eq:
        return differs(expected, reading)
    if check.op is ast.NotEq:
        return None if differs(expected, reading) is not None else "they are equal"
    assert check.op is not None
    symbol, holds = OPS[check.op]
    return None if holds(left, right) else f"`{short(left)} {symbol} {short(right)}` does not hold"


def expected_text(check: Check, expected: object) -> str:
    """What a failure shows as expected: the value, or what the shape of the check asks."""
    if check.form == "not":
        return "empty or false"
    if check.form == "isinstance":
        return f"an instance of {check.kind_text}"
    if check.form in AFFIXES:
        return f"to {'start' if check.form == 'startswith' else 'end'} with {short(expected)}"
    return f"{short(expected)} ({kind(expected)})"


def emit(fd: int, **result: object) -> None:
    if fd >= 0:  # a fork that reads one check alone says nothing but that check's verdict
        os.write(fd, (_json_dumps(result) + "\n").encode())


def frames(error: BaseException, file: str) -> list[str]:
    shown = [
        f
        for f in traceback.extract_tb(error.__traceback__)
        if os.path.realpath(f.filename) == os.path.realpath(file)
    ]
    return [f"at {rel(f.filename)}:{f.lineno}  {f.line}" for f in shown[-2:]]


def check_one(
    shape: Shape, check: Check, pair: Callable[[], Any], message: Callable[[], Any]
) -> dict[str, Any]:
    """Read one value and compare it, in this process: only the verdict, and the two texts of a
    failure, cross back."""
    try:
        left, right = pair()
        asks = message()
        if check.form in AFFIXES:  # a reading with no `startswith` raises here
            why = verdict(check, left, right)
    except Exception as error:
        return dict(
            check=check.label,
            line=check.line,
            status="fail",
            why=f"reading it raised {type(error).__name__}: {error}",
            trace=frames(error, shape.file),
        )
    if check.form not in AFFIXES:
        why = verdict(check, left, right)
    reading, expected = (left, right) if check.reading_left else (right, left)
    if why is None:
        return dict(check=check.label, line=check.line, status="pass", reading=short(reading))
    if not why and check.form == "compare" and short(expected) == short(reading):
        why = f"same text, types {kind(expected)} and {kind(reading)}"
    return dict(
        check=check.label,
        line=check.line,
        status="fail",
        asks=asks,
        expected=expected_text(check, expected),
        reading=f"{short(reading)} ({kind(reading)})",
        why=why,
    )


def read_guarded(
    run: Run, shape: Shape, check: Check, pair: Callable[[], Any], message: Callable[[], Any]
) -> dict[str, Any]:
    """One check, with each stock of the case watched: a fingerprint before and after it, or else a
    checkpoint that is restored after it. A check whose reading changed a stock is refused, by the
    stock, not by the name it reads."""
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
            return dict(
                check=check.label,
                line=check.line,
                status="refused",
                why=f"reading it changed the stock {stock_type.__name__}: a check reads what the "
                "act made, and changes nothing",
            )
    return result


def fork_alone(run: Run, shape: Shape, first: int) -> bool:
    """Before the first check reads, fork once for each later check, one at a time. Each fork runs
    the rest of the test and reads only its own check, so it reads that check alone. True in a fork,
    which then goes on with the test."""
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
            run.alone[j] = _json_loads(data.decode().splitlines()[-1])
    return False


def is_handle(value: object) -> bool:
    if isinstance(value, mmap.mmap):
        return True
    try:
        return isinstance(getattr(value, "fileno")(), int)
    except Exception:
        return False


def module_names(test_module: str) -> dict[tuple[str, str], object]:
    """Each attribute of each module the case has imported, but the test module, the form and the
    runner: what the act may replace."""
    skip = {test_module, "cases", "__main__", __name__}
    found: dict[tuple[str, str], object] = {}
    for name, module in list(sys.modules.items()):
        if name in skip or not isinstance(module, types.ModuleType):
            continue
        for attr, value in list(vars(module).items()):
            if attr.startswith("_"):
                continue  # a private global, such as a cache a library fills on first use
            found[(name, attr)] = value  # held, so no other object can take its id
    return found


def report_replaced(run: Run) -> None:
    """At the first check, before any fork for a check: each module attribute whose object is not
    the one it held as the act began. Reported, never refused; a replacement undone before this
    point is not seen."""
    before, run.before_act = run.before_act, None
    if before is None:
        return
    replaced = []
    for (name, attr), value in before.items():
        module = sys.modules.get(name)
        now = vars(module).get(attr, value) if isinstance(module, types.ModuleType) else value
        if id(now) != id(value):
            replaced.append(f"{name}.{attr}")
    for name in replaced[:3]:
        emit(run.fd, check="note", line=0, status="note", why=f"the act replaced {name}")
    if len(replaced) > 3:
        emit(run.fd, check="note", line=0, status="note", why=f"and {len(replaced) - 3} more")


def hook(test: str, k: int, pair: Callable[[], Any], message: Callable[[], Any]) -> None:
    """A check, where the test wrote its assert. On a replay it only compares; in the case, every
    check runs."""
    run = RUN[-1]
    caller = sys._getframe(1)
    shape = next(
        (
            s
            for s in run.shapes.values()
            if s.name == test
            and os.path.realpath(s.file) == os.path.realpath(caller.f_code.co_filename)
        ),
        None,
    )
    if shape is None:  # a stage of another test module, replayed
        shape = far_shape(test, caller.f_code.co_filename)
    check = shape.checks[k]
    if not run.replaying and run.only is None:
        report_replaced(run)
    if run.kids and not run.split and not run.replaying and run.only is None:
        fork_stage(run)  # the act ended at this check: fork for the stage's checks and later stages
    if run.replaying:
        left, right = pair()
        if (why := verdict(check, left, right)) is not None:
            reading, expected = (left, right) if check.reading_left else (right, left)
            shown = short(expected) if check.form == "compare" else expected_text(check, expected)
            raise Blocked(
                f"the stage {run.replaying[-1]} fails its check `{check.label}` at line "
                f"{check.line}: expected {shown}, read {short(reading)}"
                + (f" ({why})" if why else "")
            )
        return
    if run.only is not None:  # a fork that reads one check alone: the others it passes by
        if k == run.only:
            os.write(
                run.alone_fd,
                (_json_dumps(read_guarded(run, shape, check, pair, message)) + "\n").encode(),
            )
            os._exit(0)
        return
    if (
        not run.forked and os.environ.get("FORM_ALONE", "on") == "on" and fork_alone(run, shape, k)
    ):  # off only to measure
        return  # in the fork: go on to the check it reads
    run.done.add(k)
    if run.guard:
        run.guard.verify()
    local = caller.f_locals
    readers = lambda n: sum(n in c.names for c in shape.checks)  # noqa: E731
    held = sorted(n for n in check.names if n in local and is_handle(local[n]) and readers(n) >= 2)
    if held:
        emit(
            run.fd,
            check=check.label,
            line=check.line,
            status="refused",
            why=f"{held[0]} is an open file, socket, pipe or mmap, and more than one check reads "
            "it: a fork cannot seal it",
        )
        return
    # every check reads in place, in order; each later one was also read alone, before any check
    # read
    result = read_guarded(run, shape, check, pair, message)
    alone = run.alone.get(k)
    if alone and {alone["status"], result["status"]} == {"pass", "fail"}:
        before = ", ".join(str(c.line) for c in shape.checks[:k])
        result = dict(
            check=check.label,
            line=check.line,
            status="refused",
            why=f"it {'holds' if alone['status'] == 'pass' else 'fails'} when read alone, and "
            f"{'holds' if result['status'] == 'pass' else 'fails'} after the checks before it "
            f"(line{'s' * (k > 1)} {before}): a reading before it changed what it reads, and a "
            "check reads, it does not act",
        )
    emit(run.fd, **result)


class BeforeRaised(Exception):
    pass


def resolve(name: str, given: object, run: Run, top: bool, made: dict[int, object]) -> object:
    if isinstance(given, Step):
        return replay(given.case, run)
    if not isinstance(given, Use):
        return given
    factory = given.factory
    if isinstance(factory, Case):  # use(stage) is the stage's product, its checks run first
        return replay(factory, run)
    if isinstance(factory, type) and issubclass(factory, Stock):
        if (
            factory not in run.stocks
        ):  # one sample or checkpoint of each stock, for the whole case and its replays
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
    if id(factory) not in made:  # each factory runs at most once in a case
        uses = {
            p.name: resolve(p.name, p.default, run, False, made)
            for p in fixture_params(factory)
            if isinstance(p.default, Use)
        }
        made[id(factory)] = factory(**uses)
    return made[id(factory)]


def is_stage(given: object) -> bool:
    return isinstance(given, Step) or (isinstance(given, Use) and isinstance(given.factory, Case))


def build(case: Case[Any], run: Run, top: bool) -> dict[str, object]:
    """The earlier stages first, as the fork tree has run them before this case starts; then each
    `@before` of the test, top to bottom; then the fixtures, in the order of the parameters."""
    given = {
        name: case.inputs[name] if name in case.inputs else p.default
        for name, p in inspect.signature(case.test).parameters.items()
    }
    made: dict[int, object] = {}
    built = {n: resolve(n, g, run, top, made) for n, g in given.items() if is_stage(g)}
    for fn in BEFORE.get(case.test, []):
        try:
            fn()
        except Exception as error:
            raise BeforeRaised(
                f"@before({name_of(fn)}) raised {type(error).__name__}: {error}"
            ) from error
    for n, g in given.items():
        if n not in built:
            built[n] = resolve(n, g, run, top, made)
    return {n: built[n] for n in given}


def replay(step: Case[Any], run: Run) -> object:
    """Run an earlier case again, once for each case, with its checks. A step that reads differently
    blocks the case."""
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


# ── a chain as a fork tree ────────────────────────────

JOBS: list[str] = []  # the run's folder, then the folder of each case's results, for this file
REAL_ALARM = signal.alarm  # before the guard wraps it: a fork's time limit is the runner's
DID = {"fail": "failed", "refused": "was refused", "blocked": "was blocked", "skipped": "was skipped"}


def job(case: Case[Any], kind: str) -> str:
    return os.path.join(JOBS[-1], f"{id(case)}.{kind}")


def results_of(case: Case[Any]) -> list[dict[str, Any]] | None:
    try:
        with open(job(case, "res")) as f:
            return [_json_loads(line) for line in f.read().splitlines()]
    except FileNotFoundError:
        return None


def outcome(results: list[dict[str, Any]]) -> str:
    """A case's status, as `report` counts it."""
    status = "pass"
    for res in results:
        if res["status"] == "note":
            continue
        if res["status"] == "skipped":
            return "skipped"
        if res["status"] == "blocked":
            status = "blocked" if status == "pass" else status
        elif res["status"] != "pass":
            status = "refused" if res["status"] == "refused" or status == "refused" else "fail"
    return status


def label_of(case: Case[Any], names: dict[int, str]) -> str:
    return names.get(id(case), f"the case at line {case.line}")


def stages_of(case: Case[Any]) -> list[Case[Any]]:
    """The earlier cases a case takes a product of, once each: `earlier.product` in its case line,
    or `use(stage)`, in the case line or as a default."""
    found: dict[int, Case[Any]] = {}
    for name, p in inspect.signature(case.test).parameters.items():
        given = case.inputs.get(name, p.default)
        if isinstance(given, Step):
            found.setdefault(id(given.case), given.case)
        elif isinstance(given, Use) and isinstance(given.factory, Case):
            found.setdefault(id(given.factory), given.factory)
    return list(found.values())


def blocked_by(stage: Case[Any], names: dict[int, str], did: str) -> str:
    return (
        f"it takes the product of {label_of(stage, names)} ({rel(stage.file)}:{stage.line}), "
        f"which {did}"
    )


def stage_gone(case: Case[Any], run: Run) -> str | None:
    """A stage that failed, was refused, or was blocked blocks each later stage. Each stage ran
    before this case, so its results are in its file."""
    for stage in stages_of(case):
        found = results_of(stage)
        if found is not None and (status := outcome(found)) != "pass":
            return blocked_by(stage, run.names, DID[status])
    return None


def block_case(fd: int, why: str) -> None:
    emit(fd, check="case", line=0, status="blocked", why=why)


def mark_refused(case: Case[Any], status: str = "refused") -> None:
    """A case refused or skipped before the run: a later stage that takes its product reads
    this."""
    with open(job(case, "res"), "a") as f:
        f.write(_json_dumps(dict(check="case", line=0, status=status, why="")) + "\n")


def open_fds() -> dict[str, str]:
    found: dict[str, str] = {}
    for fd in os.listdir("/proc/self/fd"):
        try:
            found[fd] = os.readlink(f"/proc/self/fd/{fd}")
        except OSError:
            pass  # the descriptor of the listing itself, closed by now
    return found


def fallback_reason(run: Run) -> str | None:
    """Why this stage's chain is replayed instead of forked, if it is."""
    if run.stocks:
        stocks = ", ".join(sorted(t.__name__ for t in run.stocks))
        return f"the chain reads a stock ({stocks}), whose state a fork does not copy"
    alive = [t.name for t in threading.enumerate() if t is not threading.main_thread()]
    if alive:
        return (
            f"a thread other than the main one is alive after the act ({alive[0]}), "
            "and a fork copies only the thread that forks"
        )
    opened = sorted(
        (fd, to)
        for fd, to in open_fds().items()
        if run.fds.get(fd) != to and not to.startswith("/proc/")
    )
    if opened:  # a fork shares its offset between the later stages; a replay does not
        fd, to = opened[0]
        return (
            f"the act left a file, socket or pipe open (fd {fd} → {to}), whose offset a fork "
            "shares"
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
    run.replaying.append(label_of(run.case, run.names))


def wait_case(pid: int, case: Case[Any]) -> None:
    _, status = os.waitpid(pid, 0)
    if os.WIFSIGNALED(status):
        fd = os.open(job(case, "res"), os.O_WRONLY | os.O_CREAT | os.O_APPEND)
        why = f"the case's process died: signal {os.WTERMSIG(status)} (a case gets {SECONDS} s)"
        emit(fd, check="act", line=0, status="fail", why=why)
        os.close(fd)


def block_subtree(case: Case[Any], run: Run, why: str) -> None:
    fd = os.open(job(case, "res"), os.O_WRONLY | os.O_CREAT | os.O_APPEND)
    block_case(fd, why)
    os.close(fd)
    for kid in run.tree.get(id(case), []):
        block_subtree(kid, run, blocked_by(case, run.names, DID["blocked"]))


def spawn_later(run: Run, stage: Case[Any], stopped: str | None) -> None:
    """In the stage's continuation: fork once for each later stage, which takes the product from
    this process. A stage whose own checks did not pass blocks each of them."""
    REAL_ALARM(0)
    found = results_of(stage)
    status = outcome(found) if found is not None else "pass"
    why = blocked_by(stage, run.names, DID[status]) if status != "pass" else stopped
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


# ── a case's process ──────────────────────────


def in_case(case: Case[Any], run: Run) -> None:
    signal.alarm(SECONDS)
    RUN.append(run)
    run.case = case
    run.fds = open_fds()
    shape = run.shapes[case.test]
    lost: list[str] = []
    sys.unraisablehook = lambda u: lost.append(f"{type(u.exc_value).__name__}: {u.exc_value}")
    threading.excepthook = lambda h: lost.append(
        f"in a thread, {h.exc_type.__name__}: {h.exc_value}"
    )

    def block_the_rest(why: str) -> None:
        for k, c in enumerate(shape.checks):
            if k not in run.done:
                emit(run.fd, check=c.label, line=c.line, status="blocked", why=why)

    try:
        if (gone := stage_gone(case, run)) is not None:
            return block_case(run.fd, gone)
        try:
            inputs = build(case, run, top=True)
        except Blocked as why:
            return block_case(run.fd, str(why))
        except BeforeRaised as error:
            emit(
                run.fd,
                check="before",
                line=0,
                status="fail",
                why=str(error),
                trace=frames(error.__cause__ or error, shape.file),
            )
            return block_the_rest("a @before raised")
        except Exception as error:
            emit(
                run.fd,
                check="inputs",
                line=0,
                status="fail",
                why=f"building an input raised {type(error).__name__}: {error}",
                trace=frames(error, shape.file),
            )
            return block_the_rest("an input was not built")
        for name, value in inputs.items():  # a stand-in matches the type that the test declares
            allowed = annotation_types(
                shape.params[name].annotation, vars(sys.modules[case.test.__module__])
            )
            if (
                allowed
                and isinstance(shape.params[name].default, Use)
                and name not in case.inputs
                and not fits(value, allowed)
            ):
                emit(
                    run.fd,
                    check="inputs",
                    line=0,
                    status="fail",
                    why=f"the fixture of {name} gives {a(kind(value))}, and the test declares "
                    f"{' | '.join(t.__name__ for t in allowed)}",
                )
                return block_the_rest("an input was not built")
        SEEN.clear()
        RAISED[0] = False
        error: BaseException | None = None
        product: object = None
        stopped: str | None = None  # in a continuation: why its later stages are blocked
        guard = run.guard = Guard(run)
        guard.install()
        paths = Paths(run, shape)
        with warnings.catch_warnings(record=True) as seen:
            warnings.simplefilter("always")
            guard.start()
            paths.start()
            run.before_act = module_names(case.test.__module__)
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
                if (
                    run.only is not None
                ):  # a fork that read one check alone, or never reached it: it says nothing more
                    os._exit(0)
                if not run.cont:  # no check ran: what the act replaced, at its end
                    report_replaced(run)
                guard.verify()
                guard.on = False
        if run.kids and not run.split:  # the stage reached no check, so it forks at its end
            fork_stage(run)
        if run.cont:  # block the later stages, or fork once for each
            label = run.replaying[-1]
            if stopped is None and run.swallowed is not None:
                stopped = f"{label}: {run.swallowed}"
            if stopped is None and error is not None:
                raised = f"the stage {label} raised {type(error).__name__}: {error}"
                stopped = str(error) if isinstance(error, Missed) else raised
            run.memo[id(case)] = product
            spawn_later(run, case, stopped)
            flush()
            os._exit(0)
        if (
            error is None and not run.flagged
        ):  # the paths of the act, and whether any check ran
            for why in paths.judge():
                emit(run.fd, check="path", line=0, status="refused", why=why)
            if not run.done and not shape.expects:
                emit(
                    run.fd,
                    check="checks",
                    line=0,
                    status="refused",
                    why="no check ran: end the test with `assert reading == expected`, or expect a "
                    "raise with `with raises(…):`",
                )
        for text in SEEN:
            emit(
                run.fd,
                check=text.split(" ", 1)[0],
                line=0,
                status="pass",
                reading=text.split(" ", 1)[1],
            )
        if error is not None:
            missed = isinstance(error, Missed)
            at = None if missed else setup_at(error, shape.file, shape.name, run)
            label, why = (
                (error.form, str(error))
                if isinstance(error, Missed)
                else at or ("act", f"the act raised {type(error).__name__}: {error}")
            )
            emit(
                run.fd, check=label, line=0, status="fail", why=why, trace=frames(error, shape.file)
            )
            block_the_rest("the act did not end as the test says")
        if error is None:
            block_the_rest("it never ran")
        for w in seen:
            emit(
                run.fd,
                check="warning",
                line=0,
                status="fail",
                why=f"a warning that the test does not expect: {w.category.__name__}: {w.message}",
                trace=[f"at {rel(w.filename)}:{w.lineno}"],
            )
        for text in lost:
            emit(
                run.fd,
                check="lost error",
                line=0,
                status="fail",
                why=f"an error that no code saw: {text}",
            )
    finally:
        for stock_type, (stock, value, point) in run.stocks.items():
            try:
                if isinstance(stock, Samples):
                    stock.discard(value)
                else:
                    assert isinstance(stock, Checkpoints)
                    stock.restore(value, point)
            except Exception as error:
                emit(
                    run.fd,
                    check="stock",
                    line=0,
                    status="fail",
                    why=f"{stock_type.__name__} was not discarded or restored: "
                    f"{type(error).__name__}: {error}",
                )


def case_process(case: Case[Any], run: Run) -> typing.NoReturn:
    """The body of a case's own process: its results and output go to files of its own."""
    fd = os.open(job(case, "res"), os.O_WRONLY | os.O_CREAT | os.O_APPEND)
    out = os.open(job(case, "out"), os.O_WRONLY | os.O_CREAT | os.O_APPEND)
    os.dup2(out, 1)
    os.dup2(out, 2)
    run.fd = fd
    began = time.monotonic()
    random.seed(case_seed(case, run.names))
    if run.inherited:
        MON.restart_events()  # the earlier stage's act switched off the events its lines raised
    try:
        in_case(case, run)
    except BaseException as error:  # a fault of the runner, shown as it is
        emit(fd, check="runner", line=0, status="fail", why=f"{type(error).__name__}: {error}")
    Path(job(case, "time")).write_text(f"{time.monotonic() - began:.3f}")
    flush()
    os._exit(0)


SEED = [0]  # the run's seed: `--seed N`, or picked at the start


def case_seed(case: Case[Any], names: dict[int, str]) -> int:
    """The seed of `random` in one case: from the run's seed and the case's own name, so the same
    `--seed` gives the case the same values."""
    name = names.get(id(case), name_of(case.test))
    text = f"{SEED[0]} {os.path.basename(case.file)} {name_of(case.test)} {name}"
    return int.from_bytes(hashlib.sha256(text.encode()).digest()[:8], "big")


def run_case(case: Case[Any], run: Run) -> None:
    flush()
    pid = os.fork()
    if pid == 0:
        case_process(case, run)
    wait_case(pid, case)


def read_case(case: Case[Any]) -> tuple[list[dict[str, Any]], str, float | None]:
    try:
        with open(job(case, "out"), "rb") as f:
            captured = f.read().decode(errors="replace")
    except FileNotFoundError:
        captured = ""
    try:
        took: float | None = float(Path(job(case, "time")).read_text())
    except (FileNotFoundError, ValueError):
        took = None  # it never ran in a process of its own, or its process died
    return results_of(case) or [], captured, took


def order_of(cases: list[Case[Any]]) -> list[Case[Any]]:
    """Each case after the cases it takes a product of, and otherwise in file order."""
    among, done, out = {id(c) for c in cases}, set[int](), list[Case[Any]]()
    left_ = list(cases)
    while left_:
        ready = next(
            (c for c in left_ if all(id(s) in done for s in stages_of(c) if id(s) in among)),
            left_[0],
        )
        left_.remove(ready)
        done.add(id(ready))
        out.append(ready)
    return out


def run_chains(
    cases: list[Case[Any]], shapes: dict[Any, Shape], names: dict[int, str]
) -> dict[int, str]:
    """Run each case into its files: one tree for each case that takes no product, whose later
    stages fork from it. A case with two earlier stages forks from the first and replays the
    others. Returns each case that ran by replay, because its chain fell back, with the reason."""
    order = order_of(cases)
    among = {id(c) for c in cases}
    first = {id(c): next((s for s in stages_of(c) if id(s) in among), None) for c in order}
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
        id(r): {
            id(root_of(s))
            for c in subtree(r)
            for s in [s for s in stages_of(c) if id(s) in among][1:]
        }
        - {id(r)}
        for r in roots
    }
    ran: set[int] = set()
    fell_back: dict[int, str] = {}
    # a tree runs after the trees of its cases' other stages, whose outcomes it reads
    while roots:
        root = next((r for r in roots if needs[id(r)] <= ran), roots[0])
        roots.remove(root)
        ran.add(id(root))
        run_case(root, Run(shapes, names, tree=tree, kids=list(tree.get(id(root), []))))
        members = {id(c) for c in subtree(root)}
        for c in order:  # a stage that fell back left its later stages: replay them in order
            if id(c) in members and results_of(c) is None:
                run_case(c, Run(shapes, names))
                up: Case[Any] | None = first[id(c)]
                while up is not None and not os.path.exists(job(up, "fallback")):
                    up = first[id(up)]
                why = "its earlier stage left no result"
                fell_back[id(c)] = Path(job(up, "fallback")).read_text() if up else why
    return fell_back


# ── ④ REPORT ──────────────────────────


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
    skipped: int = 0
    checks: int = 0
    files: int = 0
    slowest: tuple[float, str] | None = None  # the time of the slowest case, and its name


def show_inputs(case: Case[Any]) -> str:
    return " ".join(f"{k}={short(v)}" for k, v in case.inputs.items()) or "(none)"


def seconds(took: float) -> str:
    return f"{took:.1f} s" if took >= 1 else f"{took:.2f} s"


def report(
    name: str,
    case: Case[Any] | None,
    results: list[dict[str, Any]],
    captured: str,
    tally: Tally,
    where: str,
    took: float | None = None,
    private: list[tuple[int, str]] | None = None,
    file: str = "",
) -> str:
    if len(results) == 1 and results[0]["status"] == "blocked" and results[0]["check"] == "case":
        say(f"   ⊘ {name}  blocked: {results[0]['why']}")
        tally.blocked += 1
        return "blocked"
    if took is not None and (tally.slowest is None or took > tally.slowest[0]):
        tally.slowest = (took, f"{name} ({where})")
    notes = [f"line {line} reads the private name {n}" for line, n in private or []]
    notes += [r["why"] for r in results if r["status"] == "note"]
    results = [r for r in results if r["status"] != "note"]
    timed = [took is not None and took >= 1]  # the case's first line shows its time

    def first(text: str) -> str:
        if timed[0]:
            timed[0] = False
            return f"{text}  {seconds(took or 0)}"
        return text

    status = "pass"
    for res in sorted(results, key=lambda r: r["line"]):
        label = f"{name} › {res['check']}"
        if res["status"] == "pass":
            tally.checks += 1
            say(first(f"   ✓ {label}   {res.get('reading', '')}"))
            continue
        if res["status"] == "blocked":
            status = "blocked" if status == "pass" else status
            say(first(f"   ⊘ {label}   blocked: {res['why']}"))
            continue
        status = (
            "refused" if res["status"] == "refused" or status == "refused" else "fail"
        )  # the test's fault outranks the code's
        mark = "!" if res["status"] == "refused" else "✗"
        line = f" · assert at line {res['line']}" if res["line"] else ""
        say(first(f"   {mark} {label}   {where}{line}"))
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
    if status in ("fail", "refused") and results:
        say(
            f"       random was seeded for this case; python run.py --seed {SEED[0]} {rel(file)} "
            "repeats it"
        )
    for text in notes:
        say(f"     ⚠ {text}")
    tally.passed += status == "pass"
    tally.failed += status == "fail"
    tally.refused += status == "refused"
    tally.blocked += status == "blocked"
    return status


def files_in(paths: list[str]) -> list[Path]:
    found: list[Path] = []
    for p in map(Path, paths or ["."]):
        if p.is_dir():
            found += sorted(
                f
                for f in p.rglob("test_*.py")
                if not any(
                    part.startswith(".") or part == "__pycache__" for part in f.relative_to(p).parts
                )
            )
        else:
            found.append(p)
    return found


def module_of(file: str) -> types.ModuleType | None:
    real = os.path.realpath(file)
    return next(
        (
            m
            for m in list(sys.modules.values())
            if os.path.realpath(getattr(m, "__file__", None) or "") == real
        ),
        None,
    )


def single_names(scope: dict[str, Any]) -> dict[int, str]:
    bound: dict[int, list[str]] = {}
    for k, v in scope.items():
        if isinstance(v, Case):
            bound.setdefault(id(v), []).append(k)
    return {i: ns[0] for i, ns in bound.items() if len(ns) == 1}


def run_file(path: Path, tally: Tally) -> None:
    real = os.path.realpath(path)
    try:
        src = Source.read(path)
    except SyntaxError as error:
        say(f"── {rel(path)}\n   ! the file is refused: {error}")
        record(real, None, None, "file refused", [str(error)])
        tally.files += 1
        return
    sys.path.insert(0, str(path.resolve().parent))
    JOBS[1:] = [tempfile.mkdtemp(dir=JOBS[0])]  # a stage of another file runs again here
    before = len(CREATED)
    try:
        module = load(path)
    except Exception as error:
        say(
            f"── {rel(path)}\n   ! the file is refused: importing it raised "
            f"{type(error).__name__}: {error}\n     so none of its tests runs"
        )
        record(real, None, None, "file refused", [f"{type(error).__name__}: {error}"])
        tally.files += 1
        return
    scope = vars(module)
    tests = sorted(
        (
            v
            for k, v in scope.items()
            if k.startswith("test")
            and isinstance(v, types.FunctionType)
            and os.path.realpath(v.__code__.co_filename) == real
        ),
        key=lambda f: f.__code__.co_firstlineno,
    )
    cases = [c for c in CREATED[before:] if os.path.realpath(c.file) == real] or [
        c for c in CREATED if os.path.realpath(c.file) == real
    ]
    if not tests and not cases:
        return
    say(f"── {rel(path)}")
    for name, lines in src.bindings.items():  # two tests with one name: Python keeps the last
        if (
            name.startswith("test")
            and len(lines) > 1
            and isinstance(scope.get(name), types.FunctionType)
        ):
            say(
                f"   ! {name} is defined at lines {', '.join(map(str, lines))}; Python keeps only "
                f"the last, so the test at line {lines[0]} would be lost"
            )
            tally.refused += len(lines) - 1
    # A case's name is read from the module after import, not from its source. `case()` keeps every
    # case it built; a case that no module name holds has none (bound again later, built by a loop,
    # never bound).
    bound: dict[int, list[str]] = {}
    for k, v in scope.items():
        if isinstance(v, Case):
            bound.setdefault(id(v), []).append(k)
    names = single_names(scope)
    shapes = {t: shape_of(t, src) for t in tests}
    stopped: set[int] = set()
    unnamed: dict[int, list[Case[Any]]] = {}
    for c in cases:
        if len(bound.get(id(c), [])) > 1:
            text = (
                f"   ! a case of {name_of(c.test)} at line {c.line} has two names, "
                f"{' and '.join(bound[id(c)])}: give each case one name"
            )
            say(text)
            record(real, name_of(c.test), " and ".join(bound[id(c)]), "refused", [text])
            tally.refused += 1
            stopped.add(id(c))
        elif id(c) not in names:
            unnamed.setdefault(c.line, []).append(c)
            stopped.add(id(c))
        elif c.test not in shapes:
            say(
                f"   ! {names[id(c)]} at line {c.line} is a case of {name_of(c.test)}, which is no "
                "test of this file: a test is a function named `test…`"
            )
            tally.refused += 1
            stopped.add(id(c))
    for line, lost in unnamed.items():
        count = f"{len(lost)} cases" if len(lost) > 1 else "a case"
        text = (
            f"   ! {count} of {name_of(lost[0].test)} at line {line} "
            f"{'have' if len(lost) > 1 else 'has'} no name, so the run cannot show "
            f"{'them' if len(lost) > 1 else 'it'}: bind each case to a name of its own, `name "
            "= case(test, …)`, once"
        )
        say(text)
        for _ in lost:
            record(real, name_of(lost[0].test), f"line {line}", "refused", [text])
        tally.refused += len(lost)
    # each test's cases, or the test alone under its own name
    plan: list[tuple[Shape, list[tuple[str, Case[Any]]], bool]] = []
    for test, shape in shapes.items():
        mine = sorted((c for c in cases if c.test is test and id(c) in names), key=lambda c: c.line)
        lone = [(shape.name, Case(test, (), {}, str(path), shape.node.lineno))] if not mine else []
        plan.append((shape, [(names[id(c)], c) for c in mine] or lone, bool(mine)))
    # a stage of another test module: loaded rewritten, as this one, and run here with every rule
    far: list[tuple[Shape, list[tuple[str, Case[Any]]], bool]] = []
    far_src: dict[int, tuple[Source, dict[str, Any]]] = {}
    seen: set[int] = set()

    def visit(c: Case[Any]) -> None:
        for s in stages_of(c):
            if os.path.realpath(s.file) == real or id(s) in seen:
                continue
            seen.add(id(s))
            other = module_of(s.file)
            if other is None:
                continue
            visit(s)
            fsrc = Source.read(Path(s.file))
            fscope = vars(other)
            if s.test not in shapes:
                shapes[s.test] = shape_of(s.test, fsrc)
            fname = single_names(fscope).get(id(s), f"the case at line {s.line}")
            names[id(s)] = fname
            far_src[id(s)] = (fsrc, fscope)
            group = next((g for g in far if g[0] is shapes[s.test]), None)
            if group is None:
                far.append((shapes[s.test], [(fname, s)], True))
            else:
                group[1].append((fname, s))

    for _, run_cases, _ in plan:
        for _, c in run_cases:
            visit(c)
    if os.environ.get("FORM_PATHS", "on") == "on":  # off only to measure what the watch costs
        arm({real} | {os.path.realpath(s.file) for s in shapes.values()})
    # refuse before the run; run every case, a chain as a fork tree; then print in file order
    faults_of: dict[int, list[str]] = {}
    queue: list[tuple[str, Any]] = []
    runnable: list[Case[Any]] = []
    skipped: set[int] = set()
    for shape, run_cases, written in far + plan:
        is_far = any(id(c) in far_src for _, c in run_cases)
        whose = f", a stage of {rel(shape.file)}" if is_far else ""
        queue.append(("say", f" {shape.name}  ({rel(shape.file)}:{shape.node.lineno}){whose}"))
        if shape.faults:
            text = "   ! refused:\n       " + "\n       ".join(shape.faults)
            queue.append(("say", text))
            queue.append(("say", f"     so none of its {max(len(run_cases), 1)} case(s) runs"))
            for name, _ in run_cases:
                record(shape.file, shape.name, name, "refused", [text])
            tally.refused += max(len(run_cases), 1)
            stopped |= {id(c) for _, c in run_cases}
            continue
        needs = [n for n, p in shape.params.items() if p.default is inspect.Parameter.empty]
        if not written and needs:
            queue.append(
                (
                    "say",
                    f"   ! refused: it takes {', '.join(needs)}, and no case gives them: add "
                    f"`name = case({shape.name}, …)`",
                )
            )
            record(shape.file, shape.name, shape.name, "refused", [queue[-1][1]])
            tally.refused += 1
            continue
        for name, case in run_cases:
            csrc, cscope = far_src.get(id(case), (src, scope))
            faults = refuse_case(case, shape, csrc, cscope, shapes, written=written)
            where = f"{rel(case.file)}:{case.line}"
            if faults:
                text = f"   ! {name} refused   {where}\n       " + "\n       ".join(faults)
                queue.append(("say", text))
                record(shape.file, shape.name, name, "refused", [text])
                tally.refused += 1
                stopped.add(id(case))
                continue
            if shape.fn in SKIPS:  # a skipped case never runs
                text = f"   ○ {name}  skipped: {SKIPS[shape.fn]}"
                queue.append(("say", text))
                record(shape.file, shape.name, name, "skipped", [text])
                tally.skipped += 1
                skipped.add(id(case))
                continue
            queue.append(("case", (name, case, written, where, shape)))
            runnable.append(case)
    for c in [*cases, *(c for _, rc, _ in far + plan for _, c in rc)]:
        if id(c) in skipped:
            mark_refused(c, "skipped")
        elif id(c) in stopped:
            mark_refused(c)
    fell_back = run_chains(runnable, shapes, names)
    for kind_, item in queue:
        if kind_ == "say":
            say(item)
            continue
        name, case, written, where, shape = item
        CAPTURE.append([])
        if id(case) in fell_back:
            say(f"   ↺ {name} ran by replay: {fell_back[id(case)]}")
        results, captured, took = read_case(case)
        status = report(
            name,
            case if written else None,
            results,
            captured,
            tally,
            where,
            took,
            shape.private,
            case.file,
        )
        record(shape.file, shape.name, name, status, CAPTURE.pop())


def main(argv: list[str]) -> int:
    tally = Tally()
    if "--seed" in argv:  # `--seed N` repeats the values of `random` in each case of a run
        at = argv.index("--seed")
        SEED[0] = int(argv[at + 1])
        argv = argv[:at] + argv[at + 2 :]
    else:
        SEED[0] = random.SystemRandom().randrange(10**6)
    JOBS.append(tempfile.mkdtemp(prefix="form-"))
    try:
        for path in files_in(argv):
            run_file(path, tally)
    finally:
        shutil.rmtree(JOBS[0])
    if summary := os.environ.get("FORM_SUMMARY"):
        Path(summary).write_text("".join(_json_dumps(r) + "\n" for r in SUMMARY))
    total = tally.passed + tally.failed + tally.refused + tally.blocked
    total = total + tally.skipped
    files = f" · {tally.files} file{'s' * (tally.files != 1)} refused" if tally.files else ""
    skipped = f" · {tally.skipped} skipped" if tally.skipped else ""
    slowest = (
        f" · slowest {tally.slowest[1]} {seconds(tally.slowest[0])}" if tally.slowest else ""
    )
    say(
        f"══ {total} case{'s' * (total != 1)}: {tally.passed} passed · "
        f"{tally.failed} failed · {tally.refused} refused · {tally.blocked} blocked{skipped}{files}"
        f"   ({tally.checks} checks held) · seed {SEED[0]}{slowest}"
    )
    every = tally.passed and tally.passed + tally.skipped == total
    return 0 if every and not tally.files else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
