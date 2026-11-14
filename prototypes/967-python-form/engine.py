"""PROTOTYPE (#967). A mini runner that every form feeds. Never merge; not how purba would build it.

Each form (stacked, klass, below) turns its own syntax into the same `Experiment` and `Case`.
So the screen is the same for every form, and only the tester's code differs.

What it applies, from the answers of #963-#966:
  parse  : the act is `return f(inputs)`; an assay is one read; each case names a value for every assay
  build  : a fork for each case; fixtures build each `use(...)` once per case; a stock is sampled or checkpointed
  replay : a case of a later step reruns the acts and the assays of the earlier steps
  react  : an unexpected warning fails; an expected raise is a reading
  assay  : a fork for each assay but the last; a stock reader runs in place between a checkpoint and a restore
  report : expected, reading, inputs
"""

from __future__ import annotations

import ast
import difflib
import inspect
import json
import os
import sys
import textwrap
import traceback
import warnings
from dataclasses import dataclass, field
from typing import Any, Callable, Generic, TypeVar, overload

T = TypeVar("T")
LITERALS = (str, int, float, bool, bytes, type(None))
OPTS = {"literals": False}


# ── what a tester writes in every form ────────────────────────────────────────


class Stock(Generic[T]):
    """The fixture of a stock. `open` is required; sample/discard, checkpoint/restore and fingerprint are optional."""

    def open(self) -> T:
        raise NotImplementedError


@dataclass
class Use:
    fixture: Any


@overload
def use(fixture: type[Stock[T]]) -> T: ...
@overload
def use(fixture: Callable[[], T]) -> T: ...
def use(fixture: Any) -> Any:
    return Use(fixture)  # type: ignore[return-value]


@dataclass
class Raises:
    type: type[BaseException]
    message: str


def raises(type_: type[BaseException], message: str) -> Raises:
    return Raises(type_, message)


@dataclass
class Warns:
    type: type[Warning]
    message: str


def warns(type_: type[Warning], message: str) -> Warns:
    return Warns(type_, message)


@dataclass
class Case:
    name: str
    given: dict[str, Any]
    expect: dict[str, Any]
    raises: Raises | None = None
    warns: Warns | None = None
    line: int = 0
    exp: Experiment | None = None


@dataclass
class After:
    """A step of a chain. `step` is a Case, or (experiment, case name) when the form names it by text."""

    step: Any
    input: str | None = None
    line: int = 0


@dataclass
class Experiment:
    name: str
    act: Callable[..., Any]
    file: str
    line: int
    assays: dict[str, Callable[..., Any]] = field(default_factory=dict)
    cases: list[Case] = field(default_factory=list)
    allow: dict[str, str] = field(default_factory=dict)
    refused: list[str] = field(default_factory=list)

    @property
    def params(self) -> list[inspect.Parameter]:
        return list(inspect.signature(self.act).parameters.values())


REGISTRY: list[Experiment] = []


def caller_line(depth: int = 2) -> int:
    return sys._getframe(depth).f_lineno


# ── parse: nothing runs yet ───────────────────────────────────────────────────


class _Unself(ast.NodeTransformer):
    """A method form writes `self.cart`; the checks read it as the input `cart`."""

    def visit_Attribute(self, node: ast.Attribute) -> ast.expr:
        if isinstance(node.value, ast.Name) and node.value.id == "self":
            return ast.copy_location(ast.Name(node.attr, ast.Load()), node)
        self.generic_visit(node)
        return node


def _src(fn: Callable[..., Any]) -> ast.FunctionDef:
    fn = getattr(fn, "__purba_src__", fn)
    tree = ast.parse(textwrap.dedent(inspect.getsource(fn)))
    node = tree.body[0]
    assert isinstance(node, ast.FunctionDef), "a function"
    if node.args.args and node.args.args[0].arg == "self":
        node = _Unself().visit(node)
    return node


def _body(fdef: ast.FunctionDef) -> list[ast.stmt]:
    body = list(fdef.body)
    if body and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant):
        body = body[1:]
    return body


def _dotted(node: ast.expr) -> list[str] | None:
    parts: list[str] = []
    while isinstance(node, ast.Attribute):
        parts.append(node.attr)
        node = node.value
    if isinstance(node, ast.Name):
        return [node.id, *reversed(parts)]
    return None


def check_act(e: Experiment) -> None:
    names = {p.name for p in e.params}
    body = _body(_src(e.act))
    if len(body) != 1 or not isinstance(body[0], ast.Return) or not isinstance(body[0].value, ast.Call):
        e.refused.append("the act is ONE call: `return f(inputs)`")
        return
    call = body[0].value
    func = _dotted(call.func)
    if func is None:
        e.refused.append(f"the act calls `{ast.unparse(call.func)}`: a name or a method of an input")
    elif any(p.startswith("_") for p in func) and "private-name" not in e.allow:
        e.refused.append(f"the act calls the private name `{'.'.join(func)}`")
    for arg in [*call.args, *(k.value for k in call.keywords)]:
        if isinstance(arg, ast.Name) and arg.id in names:
            continue
        if isinstance(arg, ast.Constant):
            if not OPTS["literals"]:
                e.refused.append(f"the act passes the literal `{ast.unparse(arg)}`: make it an input, or a default of the act")
            continue
        if isinstance(arg, ast.Call):
            e.refused.append(f"the act nests the call `{ast.unparse(arg)}`: one act per reaction")
            continue
        e.refused.append(f"the act passes `{ast.unparse(arg)}`, which is not an input")


def _read_ok(node: ast.expr, param: str) -> bool:
    if isinstance(node, ast.Name):
        return node.id == param
    if isinstance(node, ast.Attribute):
        return _read_ok(node.value, param)
    if isinstance(node, ast.Call):
        if isinstance(node.func, ast.Attribute) and not node.args and not node.keywords:
            return _read_ok(node.func.value, param)
        if isinstance(node.func, ast.Name) and len(node.args) == 1 and not node.keywords:
            return _read_ok(node.args[0], param)  # len(product)
    return False


def check_assay(e: Experiment, name: str, fn: Callable[..., Any]) -> None:
    ps = [fn.__purba_reads__] if hasattr(fn, "__purba_reads__") else list(inspect.signature(fn).parameters)
    inputs = {p.name for p in e.params}
    if len(ps) != 1 or (ps[0] != "product" and ps[0] not in inputs):
        e.refused.append(f"assay `{name}` reads one value: `product` or an input of the act")
        return
    body = _body(_src(fn))
    if len(body) != 1 or not isinstance(body[0], ast.Return) or body[0].value is None or not _read_ok(body[0].value, ps[0]):
        src = ast.unparse(body[0]) if body else "nothing"
        e.refused.append(f"assay `{name}` is one read of `{ps[0]}`, not `{src}`")


def reads(fn: Callable[..., Any]) -> str:
    return getattr(fn, "__purba_reads__", None) or next(iter(inspect.signature(fn).parameters))


def all_assays(e: Experiment) -> dict[str, Callable[..., Any]]:
    out = dict(e.assays)
    out.setdefault("product", PRODUCT)
    return out


def PRODUCT(product: Any) -> Any:  # the built-in assay: the product as it is
    return product


def check_cases(e: Experiment, stocks: set[str]) -> None:
    params = e.params
    inputs = {p.name for p in params}
    required = {p.name for p in params if p.default is inspect.Parameter.empty}
    fixed = {p.name for p in params if isinstance(p.default, Use)}
    assays = all_assays(e)
    used = set()
    for c in e.cases:
        where = f"case `{c.name}` (line {c.line})"
        for k in c.given:
            if k not in inputs:
                hint = difflib.get_close_matches(k, inputs, 1)
                e.refused.append(f"{where} gives `{k}`, which is no input" + (f"; did you mean `{hint[0]}`?" if hint else ""))
            elif k in fixed and not isinstance(c.given[k], After):
                e.refused.append(f"{where} gives `{k}`, which `use(...)` builds")
        for k in required - set(c.given):
            e.refused.append(f"{where} gives no value for the input `{k}`")
        for k in c.expect:
            if k not in assays:
                hint = difflib.get_close_matches(k, assays, 1)
                e.refused.append(f"{where} expects `{k}`, which is no assay" + (f"; did you mean `{hint[0]}`?" if hint else ""))
            used.add(k)
        for k, fn in e.assays.items():
            r = reads(fn)
            if c.raises and r == "product":
                if k in c.expect:
                    e.refused.append(f"{where} expects a raise, so assay `{k}` has no product to read")
                continue
            if k not in c.expect:
                e.refused.append(f"{where} gives no expected value for the assay `{k}`")
            v = c.given.get(r)
            if r in c.given and isinstance(v, LITERALS + (tuple,)) and not isinstance(v, After):
                e.refused.append(f"{where}: assay `{k}` reads `{r}`, a {type(v).__name__} that the act cannot change")
        if c.raises and "product" in c.expect:
            e.refused.append(f"{where} expects a raise, so `product` has nothing to read")
        if not c.expect and not c.raises:
            e.refused.append(f"{where} checks nothing: no assay and no raise")
        for k, v in c.given.items():
            if isinstance(v, After):
                check_after(e, c, k, v)
    stock_readers = [k for k, fn in e.assays.items() if reads(fn) in stocks]
    if len(stock_readers) > 1:
        pass  # the stock here offers a checkpoint, so two readers are fine


def resolve_step(a: After) -> Case | str:
    if isinstance(a.step, Case):
        return a.step
    target, name = a.step  # by text: (experiment's act or Experiment, case name)
    exp = next((x for x in REGISTRY if x.act is target or x is target or x.name == target), None)
    if exp is None:
        return f"names `{getattr(target, '__name__', target)}`, which is no experiment"
    case = next((c for c in exp.cases if c.name == name), None)
    if case is None:
        hint = difflib.get_close_matches(name, [c.name for c in exp.cases], 1)
        return f"names the case `{name}` of `{exp.name}`, which has no such case" + (f"; did you mean `{hint[0]}`?" if hint else "")
    return case


def check_after(e: Experiment, c: Case, key: str, a: After) -> None:
    step = resolve_step(a)
    if isinstance(step, str):
        e.refused.append(f"case `{c.name}` (line {c.line}): `{key}` {step}")
        return
    if a.input is None and step.raises:
        e.refused.append(f"case `{c.name}`: `{key}` takes the product of `{step.name}`, which raised; name an input")


# ── run: a fork for each case ─────────────────────────────────────────────────


class Blocked(Exception):
    pass


def literal_type(v: Any) -> bool:
    if isinstance(v, LITERALS):
        return True
    if isinstance(v, (list, tuple, set, frozenset)):
        return all(literal_type(x) for x in v)
    if isinstance(v, dict):
        return all(literal_type(k) and literal_type(x) for k, x in v.items())
    return False


def compare(expected: Any, reading: Any) -> dict[str, Any]:
    if not literal_type(reading):
        return {"ok": False, "why": f"the reading is a {type(reading).__name__}, and no literal equals it", "reading": repr(reading)}
    return {"ok": reading == expected, "reading": repr(reading), "expected": repr(expected)}


def in_fork(fn: Callable[[], dict[str, Any]]) -> dict[str, Any]:
    r, w = os.pipe()
    pid = os.fork()
    if pid == 0:
        os.close(r)
        try:
            out = fn()
        except BaseException as exc:  # noqa: BLE001
            out = {"ok": False, "why": f"the assay raised {type(exc).__name__}: {exc}"}
        os.write(w, json.dumps(out).encode())
        os._exit(0)
    os.close(w)
    data = b""
    while chunk := os.read(r, 65536):
        data += chunk
    os.close(r)
    os.waitpid(pid, 0)
    return json.loads(data or b'{"ok": false, "why": "the assay process ended"}')


class CaseRun:
    def __init__(self) -> None:
        self.built: dict[Any, Any] = {}  # one value for each use(...) in the case
        self.stocks: list[tuple[Stock[Any], Any, Any, Any]] = []
        self.replayed: list[str] = []

    def build(self, fixture: Any) -> Any:
        if fixture in self.built:
            return self.built[fixture]
        if isinstance(fixture, type) and issubclass(fixture, Stock):
            st = fixture()
            raw = st.open()
            fp = st.fingerprint(raw) if hasattr(st, "fingerprint") else None
            if hasattr(st, "sample"):
                value, mark = st.sample(raw), None
            else:
                value, mark = raw, st.checkpoint(raw)
            self.stocks.append((st, value, mark, fp))
        else:
            value = fixture()
        self.built[fixture] = value
        return value

    def end(self) -> list[str]:
        fails = []
        for st, value, mark, fp in self.stocks:
            if hasattr(st, "sample"):
                st.discard(value)
                after = st.fingerprint(st.open()) if fp is not None else None
            else:
                st.restore(value, mark)
                after = st.fingerprint(value) if fp is not None else None
            if fp is not None and fp != after:
                fails.append(f"{type(st).__name__} not restored: fingerprint {fp} ─► {after}")
        return fails

    def inputs(self, e: Experiment, c: Case) -> dict[str, Any]:
        out = {}
        for p in e.params:
            if p.name in c.given:
                v = c.given[p.name]
                out[p.name] = self.replay(v) if isinstance(v, After) else v
            elif isinstance(p.default, Use):
                out[p.name] = self.build(p.default.fixture)
            else:
                out[p.name] = p.default
        return out

    def replay(self, a: After) -> Any:
        step = resolve_step(a)
        assert isinstance(step, Case) and step.exp is not None, "checked at parse"
        e = step.exp
        args = self.inputs(e, step)
        product, raised, _w = react(e, args)
        label = f"{e.name} › {step.name}"
        got = _raise_text(raised)
        if step.raises:
            if not _raise_ok(step.raises, raised):
                raise Blocked(f"{label} diverged on replay: expected raise {step.raises.type.__name__}, got {got or 'no raise'}")
        elif raised is not None:
            raise Blocked(f"{label} raised on replay: {got}")
        for k, out in run_assays(e, step, product, args, raised is not None):
            if not out["ok"]:
                raise Blocked(f"{label} diverged on replay\n        {k}  expected {out.get('expected')}\n        {' ' * len(k)}  reading  {out.get('reading')}")
        self.replayed.append(label)
        return args[a.input] if a.input else product


def _raise_text(exc: BaseException | None) -> str:
    return f"{type(exc).__name__}: {exc}" if exc else ""


def _raise_ok(want: Raises, exc: BaseException | None) -> bool:
    return exc is not None and isinstance(exc, want.type) and want.message in str(exc)


def react(e: Experiment, args: dict[str, Any]) -> tuple[Any, BaseException | None, list[warnings.WarningMessage]]:
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        try:
            return e.act(**args), None, list(caught)
        except Exception as exc:  # noqa: BLE001
            return None, exc, list(caught)


def run_assays(e: Experiment, c: Case, product: Any, args: dict[str, Any], raised: bool) -> list[tuple[str, dict[str, Any]]]:
    stock_inputs = {p.name for p in e.params if isinstance(p.default, Use) and isinstance(p.default.fixture, type) and issubclass(p.default.fixture, Stock)}
    todo = [(k, fn) for k, fn in all_assays(e).items() if k in c.expect]
    if raised:
        todo = [(k, fn) for k, fn in todo if reads(fn) != "product"]
    inside = [(k, fn) for k, fn in todo if reads(fn) not in stock_inputs]
    stock = [(k, fn) for k, fn in todo if reads(fn) in stock_inputs]
    results = []
    for i, (k, fn) in enumerate(inside):
        val = product if reads(fn) == "product" else args[reads(fn)]
        job = lambda fn=fn, val=val, k=k: compare(c.expect[k], fn(val))  # noqa: E731
        results.append((k, job() if i == len(inside) - 1 and not stock else in_fork(job)))
    for k, fn in stock:
        db = args[reads(fn)]
        owner = next(st for st, v, *_ in CURRENT.stocks if v is db)
        mark = owner.checkpoint(db) if hasattr(owner, "checkpoint") else None
        results.append((k, compare(c.expect[k], fn(db))))
        if mark is not None:
            owner.restore(db, mark)
    order = list(c.expect)
    return sorted(results, key=lambda r: order.index(r[0]))


CURRENT: CaseRun = CaseRun()


def run_case(e: Experiment, c: Case) -> dict[str, Any]:
    global CURRENT
    CURRENT = cr = CaseRun()
    try:
        args = cr.inputs(e, c)
    except Blocked as b:
        return {"blocked": str(b)}
    shown = {k: v for k, v in args.items() if k in c.given and not isinstance(c.given[k], After)}
    product, raised, caught = react(e, args)
    out: dict[str, Any] = {"inputs": ", ".join(f"{k}={v!r}" for k, v in shown.items()), "rows": [], "replayed": cr.replayed}
    for w in caught:
        if c.warns and isinstance(w.message, c.warns.type) and c.warns.message in str(w.message):
            out["rows"].append(("warns", {"ok": True, "reading": f"{w.category.__name__}: {w.message}"}))
        else:
            out["rows"].append(("warns", {"ok": False, "why": f"an unexpected {w.category.__name__}: {w.message}"}))
            out["fail_reaction"] = True
    if c.warns and not any(r[0] == "warns" for r in out["rows"]):
        out["rows"].append(("warns", {"ok": False, "why": f"expected {c.warns.type.__name__}, none came"}))
    if c.raises:
        ok = _raise_ok(c.raises, raised)
        out["rows"].append(("raises", {"ok": ok, "reading": _raise_text(raised) or "no raise", "expected": f"{c.raises.type.__name__}: {c.raises.message}"}))
        if not ok:
            out["fail_reaction"] = True
    elif raised is not None:
        out["rows"].append(("act", {"ok": False, "why": f"the act raised {_raise_text(raised)}"}))
        out["fail_reaction"] = True
    if not out.get("fail_reaction"):
        out["rows"] += run_assays(e, c, product, args, raised is not None)
    else:
        out["rows"].append(("…", {"ok": None, "why": "assays blocked: the reaction failed"}))
    out["end"] = cr.end()
    return out


def run_case_forked(e: Experiment, c: Case) -> dict[str, Any]:
    r, w = os.pipe()
    pid = os.fork()
    if pid == 0:
        os.close(r)
        try:
            res = run_case(e, c)
        except BaseException:  # noqa: BLE001
            res = {"blocked": "PROTOTYPE BUG\n" + traceback.format_exc()}
        os.write(w, json.dumps(res, default=str).encode())
        os._exit(0)
    os.close(w)
    data = b""
    while chunk := os.read(r, 65536):
        data += chunk
    os.close(r)
    os.waitpid(pid, 0)
    return json.loads(data)


# ── report ────────────────────────────────────────────────────────────────────


def report(e: Experiment, failed_steps: set[tuple[str, str]]) -> list[str]:
    lines = [f" {e.name}  ({os.path.relpath(e.file)}:{e.line})"]
    if e.refused:
        lines += [f"   ✗ REFUSED  {why}" for why in dict.fromkeys(e.refused)]
        return lines
    for c in e.cases:
        dep = next((v for v in c.given.values() if isinstance(v, After)), None)
        step = resolve_step(dep) if dep else None
        if isinstance(step, Case) and step.exp and (step.exp.name, step.name) in failed_steps:
            lines.append(f"   ⊘ {c.name}  blocked: its step {step.exp.name} › {step.name} failed")
            failed_steps.add((e.name, c.name))
            continue
        res = run_case_forked(e, c)
        if "blocked" in res:
            lines.append(f"   ⊘ {c.name}  blocked")
            lines += [f"       {x}" for x in res["blocked"].splitlines()]
            failed_steps.add((e.name, c.name))
            continue
        for label in res["replayed"]:
            lines.append(f"   ↻ {c.name}  replayed {label}")
        bad = False
        for k, o in res["rows"]:
            if o["ok"] is None:
                lines.append(f"       {o['why']}")
            elif o["ok"]:
                lines.append(f"   ✓ {c.name} › {k:<8} {o['reading']}")
            else:
                bad = True
                lines.append(f"   ✗ {c.name} › {k}")
                if "why" in o:
                    lines.append(f"       {o['why']}")
                if "expected" in o:
                    lines.append(f"       expected {o['expected']}")
                    lines.append(f"       reading  {o['reading']}")
                if res["inputs"]:
                    lines.append(f"       inputs   {res['inputs']}")
        for f in res["end"]:
            bad = True
            lines.append(f"   ✗ {c.name} › {f}")
        if bad:
            failed_steps.add((e.name, c.name))
    return lines


def parse_all(stocks: set[str]) -> None:
    for e in REGISTRY:
        for c in e.cases:
            c.exp = e
    for e in REGISTRY:
        check_act(e)
        for k, fn in e.assays.items():
            check_assay(e, k, fn)
        if not e.assays and not any(c.expect or c.raises for c in e.cases):
            e.refused.append("the experiment has no assay")
        check_cases(e, stocks)
        for c in e.cases:
            c.exp = e
