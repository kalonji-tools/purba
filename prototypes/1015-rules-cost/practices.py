"""PROTOTYPE for #1015, never merge. The tester's side of a cost-benefit for
nine practices: how many real test functions each practice would touch.
Every `def test*` counts, at module level and inside a class.
Reads files with `ast` only; imports and runs nothing. Each count is a
syntactic signal, not a proof that the test breaks the practice."""

import ast
import collections
import pathlib
import sys

root = pathlib.Path(sys.argv[1])


def dotted(node: ast.AST) -> str:
    try:
        return ast.unparse(node)
    except Exception:
        return ""


def tail(node: ast.AST) -> str:
    return dotted(node).split("(")[0].split(".")[-1]


NONDET = {
    "random",
    "randint",
    "choice",
    "shuffle",
    "uniform",
    "uuid4",
    "urandom",
    "now",
    "utcnow",
    "today",
    "time",
    "monotonic",
    "perf_counter",
    "sleep",
}
NET = {"urlopen", "create_connection", "socket", "get", "post", "request"}
NET_MODS = ("requests.", "urllib.", "http.client", "socket.", "httpx.", "aiohttp.")
PATCH = {
    "patch",
    "patch.object",
    "patch.dict",
    "freeze_time",
    "setattr",
    "setitem",
    "setenv",
    "delenv",
    "delattr",
    "context",
}
SKIPS = {
    "skip",
    "skipif",
    "skipIf",
    "skipUnless",
    "xfail",
    "expectedFailure",
    "importorskip",
    "skipTest",
}
RAISES = {"raises", "assertRaises", "assertRaisesRegex", "warns", "assertWarns", "assertWarnsRegex"}
THREADS = {"Thread", "ThreadPoolExecutor", "ProcessPoolExecutor", "Process", "Timer"}
BROAD = {
    "Exception",
    "BaseException",
    "ValueError",
    "OSError",
    "IOError",
    "EnvironmentError",
    "TypeError",
    "RuntimeError",
}


def signals(fn: ast.FunctionDef, params: set[str]) -> set[str]:
    out = set()
    for d in fn.decorator_list:
        if tail(d.func if isinstance(d, ast.Call) else d) in SKIPS:
            out.add("P14 switched off: a skip/xfail decorator")
            if (
                isinstance(d, ast.Call)
                and not d.keywords
                and len(d.args) < 2
                and tail(d.func) in {"skipif", "xfail"}
                and not d.args[1:]
            ):
                pass
            if (
                isinstance(d, ast.Call)
                and not any(k.arg == "reason" for k in d.keywords)
                and not any(
                    isinstance(a, ast.Constant) and isinstance(a.value, str) for a in d.args
                )
            ):
                out.add("P14 a skip with no reason")
            elif not isinstance(d, ast.Call):
                out.add("P14 a skip with no reason")
    calls = [n for n in ast.walk(fn) if isinstance(n, ast.Call)]
    names = {tail(c.func) for c in calls}
    full = {dotted(c.func) for c in calls}
    if names & {"skipTest", "skip", "importorskip", "xfail"} and any(
        dotted(c.func).startswith(("pytest.", "self.")) for c in calls if tail(c.func) in SKIPS
    ):
        out.add("P14 switched off: a skip inside the body")
    if names & NONDET:
        out.add("P2 reads clock / randomness / sleeps")
    if any(f.startswith(NET_MODS) for f in full) or "urlopen" in names:
        out.add("P2 P11 touches the network")
    if "sleep" in names:
        out.add("P11 sleeps")
    patching = {"mock", "mocker", "monkeypatch"} & params
    if (
        names & PATCH
        and any(
            t in dotted(c.func)
            for c in calls
            for t in ("patch", "freeze_time", "monkeypatch", "mocker")
        )
    ) or patching:
        out.add("P10 patches a name (patch / monkeypatch / mocker / freeze_time)")
    mocks = [
        c for c in calls if tail(c.func) in {"Mock", "MagicMock", "AsyncMock", "create_autospec"}
    ]
    if mocks:
        out.add("P10 builds a mock")
        if any(
            tail(c.func) != "create_autospec"
            and not any(k.arg in {"spec", "spec_set", "autospec"} for k in c.keywords)
            and not c.args
            for c in mocks
        ):
            out.add("PY4 a mock with no spec")
    for c in calls:
        if tail(c.func) in {
            "assert_called_with",
            "assert_called_once_with",
            "assert_called",
            "assert_called_once",
            "assert_not_called",
            "assert_any_call",
            "assert_has_calls",
        }:
            out.add("P10 checks an interaction (assert_called…)")
            break
    for node in ast.walk(fn):
        items = node.items if isinstance(node, ast.With) else []
        for it in items:
            e = it.context_expr
            if isinstance(e, ast.Call) and tail(e.func) in RAISES:
                out.add("PY5 expects a raise or warning")
                has_msg = any(k.arg in {"match", "msg"} for k in e.keywords) or tail(
                    e.func
                ).endswith("Regex")
                reads = it.optional_vars is not None and any(
                    isinstance(n, ast.Attribute)
                    and isinstance(n.value, ast.Name)
                    and isinstance(it.optional_vars, ast.Name)
                    and n.value.id == it.optional_vars.id
                    for n in ast.walk(fn)
                )
                if not has_msg and not reads:
                    out.add("PY5 a raise checked by type alone")
                    if e.args and tail(e.args[0]) in BROAD:
                        out.add("PY5 a BROAD raise checked by type alone")
    if names & THREADS:
        out.add("PY7 starts a thread or process")
    # P4: more than one act phase = an assert, then more act, then an assert
    body = [
        s for s in fn.body if not (isinstance(s, ast.Expr) and isinstance(s.value, ast.Constant))
    ]
    seen_assert = seen_act_after = False
    for s in body:
        is_check = isinstance(s, ast.Assert) or (
            isinstance(s, ast.Expr)
            and isinstance(s.value, ast.Call)
            and tail(s.value.func).startswith("assert")
        )
        if is_check and seen_act_after:
            out.add("P4 act, check, act, check (more than one phase)")
            break
        if is_check:
            seen_assert = True
        elif seen_assert:
            seen_act_after = True
    n_checks = sum(1 for s in ast.walk(fn) if isinstance(s, ast.Assert)) + sum(
        1
        for c in calls
        if tail(c.func).startswith("assert")
        and tail(c.func) not in {"assert_called_with", "assert_called_once_with"}
    )
    if n_checks > 5:
        out.add("P4 more than five checks")
    return out


per_project = collections.defaultdict(collections.Counter)
totals = collections.Counter()
for project in sorted(p for p in root.iterdir() if p.is_dir()):
    for path in sorted(project.rglob("*.py")):
        if not (path.name.startswith("test_") or path.name.endswith("_test.py")) or any(
            part.startswith(".") for part in path.parts
        ):
            continue
        tree = ast.parse(path.read_text(encoding="utf-8", errors="replace"))
        fns = []
        for node in tree.body:
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                fns.append(node)
            elif isinstance(node, ast.ClassDef):
                fns += [
                    m for m in node.body if isinstance(m, (ast.FunctionDef, ast.AsyncFunctionDef))
                ]
        for fn in fns:
            if not fn.name.startswith("test"):
                continue
            params = {a.arg for a in fn.args.args + fn.args.kwonlyargs}
            per_project[project.name]["tests"] += 1
            totals["tests"] += 1
            for s in signals(fn, params):
                per_project[project.name][s] += 1
                totals[s] += 1

n = totals["tests"]
print(f"test functions read: {n} in {len(per_project)} projects")
for k, v in sorted(totals.items(), key=lambda kv: kv[0]):
    if k == "tests":
        continue
    spread = " ".join(f"{p}={c[k]}" for p, c in per_project.items() if c[k])
    print(f"{v:5d}  {100 * v / n:5.1f}%  {k:62s} {spread}")
