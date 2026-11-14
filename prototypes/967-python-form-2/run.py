"""Run the experiments in files or folders, and print what each case shows. Prototype 2 (#967). Never merge.

    python run.py [path ...]     each path a file or a folder; the default is the current folder

The exit status is 0 when every case passes, and 1 when a case fails, is refused or is blocked.
"""

from __future__ import annotations

import ast
import builtins
import difflib
import importlib
import inspect
import json
import math
import os
import re
import signal
import sys
import tempfile
import textwrap
import traceback
import types
import typing
import warnings
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from experiment import CREATED, Case, Experiment, Step, Use  # noqa: E402

LIBRARY = {"run.py", "experiment.py"}
LONG = 300
SECONDS = 30

# ── ① PARSE: refuse before anything runs ────────────────────────────────────────────

LOGIC = {
    ast.If: "an `if`: make each branch a case of its own",
    ast.IfExp: "an `x if c else y`: make each branch a case of its own",
    ast.BoolOp: "an `and` or `or` picks which side runs: make each side a case of its own",
    ast.Match: "a `match`: make each branch a case of its own",
    ast.For: "a loop: write each call on a line of its own, or make each round a case",
    ast.AsyncFor: "a loop: write each call on a line of its own, or make each round a case",
    ast.While: "a loop: write each call on a line of its own, or make each round a case",
    ast.ListComp: "a comprehension is a loop: write the values out, or make each a case",
    ast.SetComp: "a comprehension is a loop: write the values out, or make each a case",
    ast.DictComp: "a comprehension is a loop: write the values out, or make each a case",
    ast.GeneratorExp: "a comprehension is a loop: write the values out, or make each a case",
    ast.Try: "a `try`: expect a raise with .raises(…), or run code inside an `except` with handling(…)",
    ast.TryStar: "a `try`: expect a raise with .raises(…), or run code inside an `except` with handling(…)",
    ast.With: "a `with` can swallow a raise: call what it would set up, or make it a fixture with use(…)",
    ast.AsyncWith: "a `with` can swallow a raise: call what it would set up, or make it a fixture with use(…)",
    ast.Assert: "an `assert`: an assay checks, so return the value and expect it in the case",
    ast.Raise: "a `raise`: expect the code's own raise with .raises(…)",
    ast.Lambda: "a lambda: define it beside the cases, and pass it as an input",
    ast.FunctionDef: "a function inside: define it beside the cases, and pass it as an input",
    ast.AsyncFunctionDef: "a function inside: define it beside the cases, and pass it as an input",
    ast.ClassDef: "a class inside: define it beside the cases, and pass it as an input",
    ast.Global: "a `global`: pass the value as an input",
    ast.Nonlocal: "a `nonlocal`: pass the value as an input",
    ast.Yield: "a `yield`: return the product",
    ast.YieldFrom: "a `yield`: return the product",
}


def function_node(fn: Any) -> tuple[ast.FunctionDef, int]:
    lines, start = inspect.getsourcelines(fn)
    tree = ast.parse(textwrap.dedent("".join(lines)))
    node = tree.body[0]
    assert isinstance(node, ast.FunctionDef)
    return node, start - 1


def body_of(node: ast.FunctionDef) -> list[ast.stmt]:
    body = node.body
    if body and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant) and isinstance(body[0].value.value, str):
        body = body[1:]
    return body


def logic_in(stmts: list[ast.stmt], offset: int) -> str | None:
    for stmt in stmts:
        for node in ast.walk(stmt):
            why = LOGIC.get(type(node))
            if why:
                return f"line {getattr(node, 'lineno', 0) + offset} holds {why}"
    return None


def hint(word: str, choices: list[str]) -> str:
    close = difflib.get_close_matches(word, choices, n=1)
    return f"; did you mean {close[0]!r}?" if close else f"; it has {', '.join(map(repr, choices)) or 'none'}"


def refuse_experiment(exp: Experiment[Any, Any]) -> str | None:
    node, offset = function_node(exp.act)
    stmts = body_of(node)
    for stmt in stmts[:-1]:
        if isinstance(stmt, ast.Return):
            return f"line {stmt.lineno + offset} returns before the end: the act is a straight line, and `return` is its last line"
    why = logic_in(stmts, offset)
    if why:
        return f"the act {exp.name} is a straight line of calls; {why}"
    inputs = list(inspect.signature(exp.act).parameters)
    for name, read in exp.assays.items():
        params = list(inspect.signature(read).parameters)
        if len(params) != 1:
            return f"the assay {name} takes {len(params)} parameters; an assay reads one value: `product`, or one input"
        if params[0] != "product" and params[0] not in inputs:
            return f"the assay {name} reads {params[0]!r}, which is no input of {exp.name}{hint(params[0], ['product', *inputs])}"
        anode, aoffset = function_node(read)
        astmts = body_of(anode)
        if len(astmts) != 1 or not isinstance(astmts[0], ast.Return):
            return f"the assay {name} is one `return` of what it reads"
        why = logic_in(astmts, aoffset)
        if why:
            return f"the assay {name} is one `return` of what it reads; {why}"
    return None


def annotation_types(annotation: Any, scope: dict[str, Any]) -> tuple[type, ...] | None:
    """The run-time types an annotation allows, or None when it is not read (a stub-only name, Any, a TypeVar)."""
    if annotation is inspect.Parameter.empty:
        return None
    if isinstance(annotation, str):
        try:
            annotation = eval(annotation, {"__builtins__": builtins.__dict__}, {})  # builtins only: a stub-only name stays unread
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
    for kind in allowed:
        if kind is int and type(value) is bool:
            continue
        if isinstance(value, kind):
            return True
    return False


LITERAL_TYPES = (str, bytes, int, float, bool, type(None), list, tuple, dict, set, frozenset)


def literal_or_why(node: ast.expr, src: Source) -> str | None:
    if isinstance(node, ast.Constant):
        return None
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, (ast.USub, ast.UAdd)) and isinstance(node.operand, ast.Constant):
        return None
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add) and all(isinstance(s, ast.Constant) and isinstance(s.value, str) for s in (node.left, node.right)):
        return None
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        return next((w for w in (literal_or_why(e, src) for e in node.elts) if w), None)
    if isinstance(node, ast.Dict):
        return next((w for w in (literal_or_why(e, src) for e in [*node.keys, *node.values] if e is not None) if w), None)
    if isinstance(node, ast.Name) and node.id in src.values:  # a name passes when its own binding is a literal
        return literal_or_why(src.values[node.id], src)
    return f"computes `{ast.unparse(node)}`; an expected value is a literal, so the test cannot compute what the code computes"


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
            if isinstance(stmt, (ast.Assign, ast.AnnAssign)):
                for target in stmt.targets if isinstance(stmt, ast.Assign) else [stmt.target]:
                    if isinstance(target, ast.Name):
                        src.bindings.setdefault(target.id, []).append(stmt.lineno)
                        if stmt.value is not None:
                            src.values[target.id] = stmt.value
        src.values = {k: v for k, v in src.values.items() if len(src.bindings[k]) == 1}
        return src

    def statement_at(self, line: int) -> ast.stmt | None:
        return next((s for s in self.tree.body if s.lineno <= line <= (s.end_lineno or s.lineno)), None)


def refuse_unnamed(case: Case[Any], src: Source) -> str:
    stmt = src.statement_at(case.line)
    if isinstance(stmt, (ast.For, ast.While)) or (stmt and any(isinstance(n, (ast.ListComp, ast.GeneratorExp, ast.DictComp)) for n in ast.walk(stmt))):
        return "is built by a loop; write each case on a line of its own, `name = experiment(…).expect(…)`"
    if isinstance(stmt, ast.Assign) and len(stmt.targets) == 1 and isinstance(stmt.targets[0], ast.Name):
        name = stmt.targets[0].id
        later = [n for n in src.bindings.get(name, []) if n > stmt.lineno]
        if later:
            return f"is named {name!r}, and line {later[0]} binds {name!r} again, so this case would be lost"
    return "has no name; bind it to a variable, `name = experiment(…).expect(…)`"


def refuse_case(case: Case[Any], src: Source, scope: dict[str, Any]) -> str | None:
    exp = case.experiment
    if case.args:
        return f"passes {len(case.args)} input(s) by position; name each input, as `{exp.name}(name=…)`"
    params = inspect.signature(exp.act).parameters
    for key in case.inputs:
        if key not in params:
            return f"gives {key!r}, which is no input of {exp.name}{hint(key, list(params))}"
    for name, p in params.items():
        if name not in case.inputs and p.default is inspect.Parameter.empty:
            return f"gives no {name!r}, an input of {exp.name} with no default"
    if not case.expected and not case.raised:
        return "checks nothing; add .expect(…) or .raises(…)"
    for key in case.expected:
        if key != "product" and key not in exp.assays:
            return f"expects {key}=…, which is no assay of {exp.name}{hint(key, ['product', *exp.assays])}"
        reads = "product" if key == "product" else next(iter(inspect.signature(exp.assays[key]).parameters))
        if case.raised and reads == "product":
            return f"expects {key}=…, which reads the product, and an act that raises makes no product"
    stmt = src.statement_at(case.line)
    for node in ast.walk(stmt) if stmt else []:
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == "expect":
            for label, value in [("product", a) for a in node.args] + [(k.arg or "**", k.value) for k in node.keywords]:
                why = literal_or_why(value, src)
                if why:
                    return f"expects {label}=…, which {why}"
    for key, value in case.expected.items():
        returns = inspect.signature(exp.act if key == "product" else exp.assays[key]).return_annotation
        allowed = annotation_types(returns, scope)
        if allowed and not fits(value, allowed):
            kinds = " | ".join(t.__name__ for t in allowed)
            fix = f": write {float(value)!r}" if type(value) is int and float in allowed else ""
            return f"expects {key}={short(value)} ({kind(value)}), and {'the act' if key == 'product' else 'the assay ' + key} returns {kinds}{fix}"
    return None


# ── ②-④ BUILD, REACT, ASSAY: one forked process for each case ───────────────────────


class Blocked(Exception):
    pass


def short(value: object) -> str:
    if callable(value) and getattr(value, "__name__", "") == "<lambda>":
        code = getattr(value, "__code__", None)
        return f"<lambda at line {code.co_firstlineno}>" if code else "<lambda>"
    if isinstance(value, types.FunctionType):
        return f"{value.__name__}"
    text = repr(value)
    return text if len(text) <= LONG else text[: LONG - 1] + "…"


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
    return None if reading == expected else ""


def resolve(value: object, memo: dict[int, object], names: dict[int, str]) -> object:
    if isinstance(value, Use):
        return value.factory()
    if isinstance(value, Step):
        return replay(value.case, memo, names)
    return value


def build(case: Case[Any], memo: dict[int, object], names: dict[int, str]) -> dict[str, object]:
    built = {}
    for name, p in inspect.signature(case.experiment.act).parameters.items():
        given = case.inputs[name] if name in case.inputs else p.default
        built[name] = resolve(given, memo, names)
    return built


def replay(step: Case[Any], memo: dict[int, object], names: dict[int, str]) -> object:
    """Run an earlier case again, ONCE per case: every input taken from it reads this one replay."""
    if id(step) in memo:
        return memo[id(step)]
    label = names.get(id(step), f"line {step.line}")
    inputs = build(step, memo, names)
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        try:
            product = step.experiment.act(**inputs)
        except Exception as error:
            raise Blocked(f"the step {label} raised {type(error).__name__}: {error}") from None
    for key, expected in step.expected.items():
        reading = product if key == "product" else read(step.experiment, key, product, inputs)
        if differs(expected, reading) is not None:
            raise Blocked(f"the step {label} replayed differently: {key} read {short(reading)}, not {short(expected)}")
    memo[id(step)] = product
    return product


def read(exp: Experiment[Any, Any], assay: str, product: object, inputs: dict[str, object]) -> object:
    fn = exp.assays[assay]
    param = next(iter(inspect.signature(fn).parameters))
    return fn(product if param == "product" else inputs[param])


def emit(fd: int, **result: object) -> None:
    os.write(fd, (json.dumps(result) + "\n").encode())


def check(fd: int, case: Case[Any], key: str, product: object, inputs: dict[str, object]) -> None:
    try:
        reading = product if key == "product" else read(case.experiment, key, product, inputs)
    except Exception as error:
        emit(fd, check=key, status="fail", why=f"the assay raised {type(error).__name__}: {error}", trace=frames(error, case))
        return
    expected = case.expected[key]
    why = differs(expected, reading)
    if why is None:
        emit(fd, check=key, status="pass", reading=short(reading))
        return
    if not why and short(expected) == short(reading):
        why = f"same text, types {kind(expected)} and {kind(reading)}"
    emit(fd, check=key, status="fail", expected=f"{short(expected)} ({kind(expected)})", reading=f"{short(reading)} ({kind(reading)})", why=why)


def frames(error: BaseException, case: Case[Any]) -> list[str]:
    mine = {os.path.realpath(case.file), os.path.realpath(inspect.getfile(case.experiment.act))}
    shown = [f for f in traceback.extract_tb(error.__traceback__) if os.path.realpath(f.filename) in mine]
    return [f"at {rel(f.filename)}:{f.lineno}  {f.line}" for f in shown[-2:]]


def in_case(fd: int, case: Case[Any], names: dict[int, str]) -> None:
    signal.alarm(SECONDS)
    keys = list(case.expected)
    memo: dict[int, object] = {}
    try:
        inputs = build(case, memo, names)
    except Blocked as why:
        for key in ["raises"] * bool(case.raised) + keys:
            emit(fd, check=key, status="blocked", why=str(why))
        return
    except Exception as error:
        emit(fd, check="inputs", status="fail", why=f"building an input raised {type(error).__name__}: {error}")
        return
    product: object = None
    raised: BaseException | None = None
    with warnings.catch_warnings(record=True) as seen:
        warnings.simplefilter("always")
        try:
            product = case.experiment.act(**inputs)
        except Exception as error:
            raised = error
    for kind_, match in case.warned:
        hit = next((w for w in seen if issubclass(w.category, kind_) and (match is None or re.search(match, str(w.message)))), None)
        if hit:
            seen.remove(hit)
            emit(fd, check="warns", status="pass", reading=f"{hit.category.__name__}: {hit.message}")
        else:
            emit(fd, check="warns", status="fail", expected=f"{kind_.__name__} matching {match!r}", reading=", ".join(f"{w.category.__name__}: {w.message}" for w in seen) or "no warning")
    for w in seen:
        emit(fd, check="warned", status="fail", why=f"a warning that the case does not expect: {w.category.__name__}: {w.message}", trace=[f"at {rel(w.filename)}:{w.lineno}"])
    if case.raised:
        kind_, match = case.raised
        if raised is None:
            emit(fd, check="raises", status="fail", expected=f"{kind_.__name__} matching {match!r}", reading=f"no raise; the act returned {short(product)}")
            for key in keys:
                emit(fd, check=key, status="blocked", why="the act did not raise")
            return
        if not isinstance(raised, kind_) or (match is not None and not re.search(match, str(raised))):
            emit(fd, check="raises", status="fail", expected=f"{kind_.__name__} matching {match!r}", reading=f"{type(raised).__name__}: {raised}", trace=frames(raised, case))
            for key in keys:
                emit(fd, check=key, status="blocked", why="the act raised something else")
            return
        emit(fd, check="raises", status="pass", reading=f"{type(raised).__name__}: {raised}")
    elif raised is not None:
        emit(fd, check="act", status="fail", why=f"the act raised {type(raised).__name__}: {raised}", trace=frames(raised, case))
        for key in keys:
            emit(fd, check=key, status="blocked", why="the act raised")
        return
    for key in keys[:-1]:  # each assay but the last in a fork of its own: none can change what another reads
        pid = os.fork()
        if pid == 0:
            check(fd, case, key, product, inputs)
            os._exit(0)
        os.waitpid(pid, 0)
    if keys:
        check(fd, case, keys[-1], product, inputs)


def run_case(case: Case[Any], names: dict[int, str]) -> tuple[list[dict[str, Any]], str]:
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
            in_case(w, case, names)
        except BaseException as error:  # a fault of the runner, shown as it is
            emit(w, check="runner", status="fail", why=f"{type(error).__name__}: {error}")
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
        results.append({"check": "act", "status": "fail", "why": f"the case's process died: signal {os.WTERMSIG(status)} (a case gets {SECONDS} s)"})
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
    return " ".join(f"{k}={short(v)}" for k, v in case.inputs.items()) or "(the defaults)"


def report(name: str, case: Case[Any], results: list[dict[str, Any]], captured: str, tally: Tally) -> None:
    status = "pass"
    for res in results:
        if res["status"] == "pass":
            tally.checks += 1
            print(f"   ✓ {name} › {res['check']}  {res.get('reading', '')}")
            continue
        if res["status"] == "blocked":
            status = "blocked" if status == "pass" else status
            print(f"   ⊘ {name} › {res['check']}  blocked: {res['why']}")
            continue
        status = "fail"
        print(f"   ✗ {name} › {res['check']}   {rel(case.file)}:{case.line}")
        doc = inspect.getdoc(case.experiment.assays[res["check"]]) if res["check"] in case.experiment.assays else None
        if doc:
            print(f"       asks      {doc}")
        for label in ("expected", "reading"):
            if label in res:
                print(f"       {label:<9} {res[label]}")
        if res.get("why"):
            print(f"       {'why' if 'expected' in res else 'what':<9} {res['why']}")
        for line in res.get("trace", []):
            print(f"                 {line}")
        print(f"       inputs    {show_inputs(case)}")
    if status != "pass" and captured.strip():
        tail = captured.strip().splitlines()[-8:]
        print("       output    " + "\n                 ".join(tail))
    if status == "pass":
        tally.passed += 1
    elif status == "fail":
        tally.failed += 1
    else:
        tally.blocked += 1


def files_in(paths: list[str]) -> list[Path]:
    found: list[Path] = []
    for p in map(Path, paths or ["."]):
        if p.is_dir():
            found += sorted(f for f in p.rglob("*.py") if f.name not in LIBRARY and not f.name.startswith("_") and "__pycache__" not in f.parts and not any(part.startswith(".") for part in f.parts))
        else:
            found.append(p)
    return found


def run_file(path: Path, tally: Tally) -> None:
    real = os.path.realpath(path)
    src = Source.read(path)
    sys.path.insert(0, str(path.resolve().parent))
    before = len(CREATED)
    try:
        module = sys.modules.get(path.stem) or importlib.import_module(path.stem)
    except Exception as error:
        print(f"── {rel(path)}\n   ! the file is refused: importing it raised {type(error).__name__}: {error}\n     so none of its cases runs")
        tally.files += 1
        return
    scope = vars(module)
    experiments = sorted(
        {id(v): v for v in scope.values() if isinstance(v, Experiment) and os.path.realpath(inspect.getfile(v.act)) == real}.values(),
        key=lambda e: inspect.getsourcelines(e.act)[1],
    )
    cases = [c for c in CREATED[before:] if os.path.realpath(c.file) == real] or [c for c in CREATED if os.path.realpath(c.file) == real]
    if not experiments and not cases:
        return
    print(f"── {rel(path)}")
    names = {id(v): k for k, v in scope.items() if isinstance(v, Case)}
    for c in cases:
        if id(c) not in names:
            print(f"   ! a case of {c.experiment.name} at line {c.line} {refuse_unnamed(c, src)}")
            tally.refused += 1
    refused_steps: set[int] = set()
    groups = [*experiments, *sorted({id(c.experiment): c.experiment for c in cases if c.experiment not in experiments}.values(), key=lambda e: e.name)]
    for exp in groups:
        mine = sorted((c for c in cases if c.experiment is exp and id(c) in names), key=lambda c: c.line)
        print(f" {exp.name}  ({rel(inspect.getfile(exp.act))}:{inspect.getsourcelines(exp.act)[1]})")
        why = refuse_experiment(exp)
        if why:
            print(f"   ! refused: {why}\n     so none of its {len(mine)} case(s) runs")
            tally.refused += len(mine)
            refused_steps |= {id(c) for c in mine}
            continue
        if not mine:
            print("   ! refused: no case calls it")
            tally.refused += 1
            continue
        for case in mine:
            name = names[id(case)]
            why = refuse_case(case, src, scope)
            if why:
                print(f"   ! {name} refused   {rel(case.file)}:{case.line}\n       {why}")
                tally.refused += 1
                refused_steps.add(id(case))
                continue
            refused_input = next((names.get(id(v.case), "?") for v in case.inputs.values() if isinstance(v, Step) and id(v.case) in refused_steps), None)
            if refused_input:
                print(f"   ⊘ {name}  blocked: it takes the product of {refused_input}, which is refused")
                tally.blocked += 1
                continue
            results, captured = run_case(case, names)
            report(name, case, results, captured, tally)


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
