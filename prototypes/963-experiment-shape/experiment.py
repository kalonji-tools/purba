"""PROTOTYPE (#963), throwaway. Not purba, and not how purba would build it.

A mini runner for the "experiment" shape agreed in the #963 grilling:

    case table  ->  inputs (built fresh per case)  ->  reaction: ONE act
                ->  product  ->  assays (each hands a reading)  ->  report

Only what it takes to feel the flow. No error handling beyond that.
"""

import ast
import copy
import inspect
import textwrap
import time
import warnings
from dataclasses import dataclass

NA = ...  # a cell that does not apply to this case


@dataclass(frozen=True)
class raises:
    type: type
    message: str  # must appear in str(exception)

    def __repr__(self) -> str:
        return f"{self.type.__name__}(…{self.message}…)"


@dataclass(frozen=True)
class warns:
    type: type
    message: str

    def __repr__(self) -> str:
        return f"{self.type.__name__}(…{self.message}…)"


@dataclass(frozen=True)
class after:
    """A chain: this input is what `name` was after `case` of `experiment`."""

    experiment: "Experiment"
    case: str
    name: str = "product"

    def __repr__(self) -> str:
        return f"<{self.experiment.name} › {self.case}>"


BUILDERS: dict[str, dict] = {}


def builder(fn):
    """A builder makes the input of the same name from the table's cell."""
    BUILDERS.setdefault(fn.__module__, {})[fn.__name__] = fn
    return fn


REGISTRY: list["Experiment"] = []


class Experiment:
    def __init__(self, act, columns, rows, allow, skip):
        self.act, self.name, self.module = act, act.__name__, act.__module__
        self.columns, self.rows, self.allow, self.skip = columns, rows, allow, skip
        self.inputs = list(inspect.signature(act).parameters)
        self.assays: dict[str, object] = {}
        self.results: dict[str, tuple[bool, dict]] = {}
        self.refused: list[str] = []

    def assay(self, fn):
        self.assays[fn.__name__] = fn
        return fn


def experiment(*, columns, allow=None, skip=None, **rows):
    def wrap(act):
        e = Experiment(act, columns, rows, allow or {}, skip or {})
        REGISTRY.append(e)
        return e

    return wrap


# ── ① PARSE: an allow list on the shape. Nothing runs. ─────────────────────────
BANNED = (ast.IfExp, ast.BoolOp, ast.ListComp, ast.SetComp, ast.DictComp,
          ast.GeneratorExp, ast.Lambda, ast.NamedExpr, ast.Await)


def _body(fn) -> list[ast.stmt]:
    tree = ast.parse(textwrap.dedent(inspect.getsource(fn)))
    body = tree.body[0].body
    if body and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant):
        body = body[1:]  # a docstring is fine
    return body


def _private(node: ast.AST) -> list[str]:
    names = []
    for n in ast.walk(node):
        name = n.id if isinstance(n, ast.Name) else n.attr if isinstance(n, ast.Attribute) else ""
        if name.startswith("_"):
            names.append(name)
    return names


def check(e: Experiment) -> None:
    say = e.refused.append
    body = _body(e.act)
    stmt = body[0] if len(body) == 1 else None
    if stmt is None or not isinstance(stmt, ast.Return) or not isinstance(stmt.value, ast.Call):
        found = ", ".join(type(s).__name__ for s in body)
        say(f"the reaction must be inputs + ONE act (`return act(inputs)`); found: {found}")
    else:
        call = stmt.value
        for arg in [*call.args, *(k.value for k in call.keywords)]:
            if not (isinstance(arg, ast.Name) and arg.id in e.inputs):
                say(f"each argument of the act must be an input; found `{ast.unparse(arg)}`")
        if (p := _private(call.func)) and "private-name" not in e.allow:
            say(f"the act calls a private name `{p[0]}` (give a reason to depart)")
    for name, fn in e.assays.items():
        body = _body(fn)
        if len(body) != 1 or not isinstance(body[0], ast.Return):
            say(f"assay `{name}` must be ONE `return <reading>`")
            continue
        for n in ast.walk(body[0]):
            if isinstance(n, BANNED):
                say(f"assay `{name}` holds logic: `{ast.unparse(n)}`")
        if (p := _private(body[0])) and "private-name" not in e.allow:
            say(f"assay `{name}` reads a private name `{p[0]}` (give a reason to depart)")
        bad = set(inspect.signature(fn).parameters) - {"product", *e.inputs}
        if bad:
            say(f"assay `{name}` reads `{bad.pop()}`, which is neither the product nor an input")
    known = {*e.inputs, *e.assays, "raises", "warns"}
    for col in e.columns:
        if col not in known:
            say(f"column `{col}` is neither an input, an assay, `raises` nor `warns`")
    for need in [*e.inputs, *e.assays]:
        if need not in e.columns:
            say(f"`{need}` has no column in the case table")
    if not e.assays and not all(_cells(e, r).get("raises", NA) is not NA for r in e.rows.values()):
        say("an experiment needs at least one assay")
    for case, row in e.rows.items():
        if len(row) != len(e.columns):
            say(f"case `{case}` has {len(row)} cells for {len(e.columns)} columns")


def _cells(e, row) -> dict:
    return dict(zip(e.columns, row))


# ── ②-⑤ BUILD, REACT, ASSAY, REPORT ────────────────────────────────────────────
OUT: list[str] = []
TALLY = {"passed": 0, "failed": 0, "blocked": 0, "skipped": 0, "refused": 0}


def out(line: str = "") -> None:
    OUT.append(line)


def _missing(value, typ) -> list[str]:
    wants = [m for m in dir(typ) if not m.startswith("_")]
    return [m for m in wants if not hasattr(value, m)]


def run_case(e: Experiment, case: str) -> None:
    if case in e.results:
        return
    cells = _cells(e, e.rows[case])
    if case in e.skip:
        out(f"    ⊘ {case}  skipped: {e.skip[case]}")
        TALLY["skipped"] += 1
        e.results[case] = (False, {})
        return

    # ② BUILD: fresh inputs; a stand-in must match the declared type.
    hints = inspect.get_annotations(e.act, eval_str=True)
    ns: dict = {}
    for name in e.inputs:
        cell = cells[name]
        if isinstance(cell, after):
            run_case(cell.experiment, cell.case)
            ok, upstream = cell.experiment.results[cell.case]
            if not ok:
                out(f"    ⊘ {case}  blocked: step `{cell.experiment.name} › {cell.case}` did not pass")
                TALLY["blocked"] += 1
                e.results[case] = (False, {})
                return
            ns[name] = copy.deepcopy(upstream[cell.name])
            continue
        make = BUILDERS.get(e.module, {}).get(name)
        ns[name] = make(copy.deepcopy(cell)) if make else copy.deepcopy(cell)
        typ = hints.get(name)
        if getattr(typ, "_is_protocol", False) and not isinstance(ns[name], typ):
            miss = ", ".join(_missing(ns[name], typ))
            out(f"    ✗ {case}  input `{name}`: {type(ns[name]).__name__} is not a {typ.__name__} (missing: {miss})")
            TALLY["failed"] += 1
            e.results[case] = (False, {})
            return

    # ③ REACT: once. The act's return value is the product.
    expect_raise, expect_warn = cells.get("raises", NA), cells.get("warns", NA)
    product, raised, ok = NA, None, True
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        try:
            product = e.act(**ns)
        except Exception as exc:  # noqa: BLE001
            raised = exc
    for w in caught:
        if expect_warn is not NA and isinstance(w.message, expect_warn.type) and expect_warn.message in str(w.message):
            out(f"    ✓ {case} › warns   {type(w.message).__name__}: {w.message}")
            expect_warn = "seen"
        else:
            out(f"    ✗ {case} › an unexpected warning   {type(w.message).__name__}: {w.message}")
            ok = False
    if expect_warn not in (NA, "seen"):
        out(f"    ✗ {case} › warns   expected {expect_warn!r}, none came")
        ok = False
    if raised is not None:
        if expect_raise is not NA and isinstance(raised, expect_raise.type) and expect_raise.message in str(raised):
            out(f"    ✓ {case} › raises  {type(raised).__name__}: {raised}")
        else:
            out(f"    ✗ {case} › the reaction raised {type(raised).__name__}: {raised}")
            out(f"        expected  {expect_raise if expect_raise is not NA else 'no raise'}")
            ok = False
    elif expect_raise is not NA:
        out(f"    ✗ {case} › raises  expected {expect_raise!r}, the act returned {product!r}")
        ok = False
    if not ok:
        blocked = [a for a in e.assays if cells.get(a, NA) is not NA]
        if blocked:
            out(f"        assays blocked: {', '.join(blocked)}")
        TALLY["failed"] += 1
        e.results[case] = (False, {})
        return

    # ④ ASSAY: every assay runs, each on its own copy (the product stays sealed).
    copies, t0 = 0, time.perf_counter()
    for name, fn in e.assays.items():
        expected = cells.get(name, NA)
        if expected is NA:
            continue
        params = list(inspect.signature(fn).parameters)
        if raised is not None and "product" in params:
            continue  # an expected raise leaves no product; assays on the inputs still run
        args = {p: copy.deepcopy(product if p == "product" else ns[p]) for p in params}
        copies += len(args)
        try:
            reading = fn(**args)
        except Exception as exc:  # noqa: BLE001
            reading = f"<assay raised {type(exc).__name__}: {exc}>"
        if reading == expected:
            out(f"    ✓ {case} › {name:<10} {reading!r}")
        else:
            ok = False
            out(f"    ✗ {case} › {name}")
            out(f"        expected  {expected!r}")
            out(f"        reading   {reading!r}")
            shown = {k: v for k, v in cells.items() if k in e.inputs}
            out(f"        inputs    {', '.join(f'{k}={v!r}' for k, v in shown.items())}")
    cost = time.perf_counter() - t0
    if cost > 0.05:
        out(f"    ⚠ {case}  sealing cost: {copies} copies of the product, {cost:.2f} s")
    TALLY["passed" if ok else "failed"] += 1
    e.results[case] = (ok, {"product": product, **ns})


def run_all() -> None:
    seen: dict = {}
    for e in REGISTRY:
        key = (e.module, e.name)
        if key in seen:
            e.refused.append(f"a second experiment named `{e.name}` in this module")
            seen[key].refused.append(f"a second experiment named `{e.name}` in this module")
        seen[key] = e
        check(e)
    module = None
    for e in REGISTRY:
        if e.module != module:
            module = e.module
            out()
            out(f"{module}.py")
        line = inspect.getsourcelines(e.act)[1]
        out(f"  {e.name}  (line {line})")
        if e.refused:
            for why in e.refused:
                out(f"    ✗ REFUSED before the run: {why}")
            TALLY["refused"] += 1
            continue
        for kind, reason in e.allow.items():
            out(f"    ⚠ departure `{kind}`: {reason}")
        for case in e.rows:
            run_case(e, case)
    out()
    out("─" * 72)
    out("  ".join(f"{n} {k}" for k, n in TALLY.items()))
    print("\n".join(OUT))
