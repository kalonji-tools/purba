"""PROTOTYPE for #1019, never merge: split every switch-off in the pinned corpus by shape.

Read only: parses each test file with ast. Usage: python -I skips.py <corpus dir>
"""

import ast
import collections
import pathlib
import sys

SKIP_DEC = {"skip", "skipif", "skipIf", "skipUnless", "xfail", "expectedFailure"}
BODY_CALLS = {"skip", "skipTest", "importorskip", "xfail"}
C = collections.Counter
by = collections.defaultdict(C)
examples = collections.defaultdict(list)


def tail(n):
    if isinstance(n, ast.Call):
        n = n.func
    if isinstance(n, ast.Attribute):
        return n.attr
    if isinstance(n, ast.Name):
        return n.id
    return ""


def cond_kind(node):
    if node is None:
        return "none"
    s = ast.unparse(node)
    low = s.lower()
    if any(k in low for k in ("platform", "os.name", "win", "darwin", "linux", "macos", "posix")):
        return "platform"
    if any(k in low for k in ("pypy", "implementation", "cpython", "jython")):
        return "interpreter"
    if any(k in low for k in ("version_info", "py2", "py3", "sys.version", "python_version")):
        return "python version"
    if any(k in low for k in ("environ", "getenv", " ci", "ci ")):
        return "environment"
    if any(
        k in low
        for k in (
            "import",
            "find_spec",
            "is none",
            "has_",
            "have_",
            "missing",
            "available",
            "installed",
            "not ",
        )
    ):
        return "optional dependency"
    return "other"


def reason_of(call, name):
    """Return 'literal', 'computed', or 'none' for the reason of a skip."""
    if not isinstance(call, ast.Call):
        return "none"
    r = None
    for k in call.keywords:
        if k.arg in ("reason", "msg"):
            r = k.value
    if r is None:
        if name in ("skip", "skipTest", "importorskip") and call.args:
            r = (
                call.args[0]
                if name != "importorskip"
                else (call.args[2] if len(call.args) > 2 else None)
            )
            if name == "importorskip" and r is None:
                return "implied (module name)"
        elif name in ("skipIf", "skipUnless") and len(call.args) > 1:
            r = call.args[1]
        elif name in ("skipif", "xfail") and len(call.args) > 1:
            r = call.args[1]
    if r is None:
        return "none"
    if isinstance(r, ast.Constant) and isinstance(r.value, str):
        return "literal" if r.value.strip() else "empty"
    if isinstance(r, ast.JoinedStr):
        return "f-string"
    return "computed"


def cond_of(call, name):
    if not isinstance(call, ast.Call):
        return None
    if name in ("skipif", "skipIf", "skipUnless") and call.args:
        return call.args[0]
    if name == "xfail" and call.args:
        a = call.args[0]
        if not (isinstance(a, ast.Constant) and isinstance(a.value, str)):
            return a
    for k in call.keywords:
        if k.arg == "condition":
            return k.value
    return None


def kind(name, call):
    if name == "skip":
        return "skip (always)"
    if name in ("skipif", "skipIf", "skipUnless"):
        return "skip if"
    if name == "xfail":
        strict = any(
            k.arg == "strict" and getattr(k.value, "value", None) is True
            for k in getattr(call, "keywords", [])
        )
        return "xfail (strict)" if strict else "xfail (not strict)"
    if name == "expectedFailure":
        return "expectedFailure"
    return name


def record(project, path, line, site, k, cond, reason):
    key = (site, k)
    by["kind"][key] += 1
    by["kind x condition"][(site, k, cond)] += 1
    by["reason"][(site, k, reason)] += 1
    by["project"][(project, site)] += 1
    if len(examples[(site, k, cond)]) < 2:
        examples[(site, k, cond)].append(f"{path}:{line}")


def aliases(tree):
    out = {}
    for st in tree.body:
        if (
            isinstance(st, ast.Assign)
            and len(st.targets) == 1
            and isinstance(st.targets[0], ast.Name)
        ):
            v = st.value
            if tail(v) in SKIP_DEC:
                out[st.targets[0].id] = v
    return out


def tests(tree):
    for st in tree.body:
        if isinstance(st, (ast.FunctionDef, ast.AsyncFunctionDef)) and st.name.startswith("test"):
            yield "module-level def", st
        elif isinstance(st, ast.ClassDef):
            for m in st.body:
                if isinstance(m, (ast.FunctionDef, ast.AsyncFunctionDef)) and m.name.startswith(
                    "test"
                ):
                    yield "class method", m


def enclosing_if(fn, target):
    for node in ast.walk(fn):
        if isinstance(node, ast.If):
            for sub in node.body:
                for n in ast.walk(sub):
                    if n is target:
                        return node.test
    return None


root = pathlib.Path(sys.argv[1])
totals = C()
for project in sorted(p.name for p in root.iterdir() if p.is_dir()):
    files = [
        f
        for f in (root / project).rglob("*.py")
        if (f.name.startswith("test_") or f.name.endswith("_test.py")) and "/." not in str(f)
    ]
    for f in sorted(files):
        try:
            tree = ast.parse(f.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        rel = f.relative_to(root)
        al = aliases(tree)
        # module-level switch-offs
        for st in tree.body:
            if isinstance(st, ast.Assign) and any(
                isinstance(t, ast.Name) and t.id == "pytestmark" for t in st.targets
            ):
                record(project, rel, st.lineno, "module", "pytestmark", cond_kind(st.value), "n/a")
            for n in (
                ast.walk(st)
                if not isinstance(st, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef))
                else []
            ):
                if (
                    isinstance(n, ast.Call)
                    and tail(n) in ("importorskip", "skip")
                    and ast.unparse(n.func).startswith("pytest.")
                ):
                    record(
                        project,
                        rel,
                        n.lineno,
                        "module",
                        "module " + tail(n),
                        cond_kind(st.test) if isinstance(st, ast.If) else "none",
                        reason_of(n, tail(n)),
                    )
                if (
                    isinstance(n, ast.Raise)
                    and n.exc is not None
                    and "SkipTest" in ast.unparse(n.exc)
                ):
                    record(
                        project,
                        rel,
                        n.lineno,
                        "module",
                        "raise SkipTest",
                        cond_kind(st.test) if isinstance(st, ast.If) else "none",
                        "n/a",
                    )
        for site, fn in tests(tree):
            totals[site] += 1
            for d in fn.decorator_list:
                name = tail(d)
                via = "direct"
                target = d
                if isinstance(d, ast.Name) and d.id in al:
                    target, name, via = al[d.id], tail(al[d.id]), "module alias"
                elif (
                    isinstance(d, (ast.Name, ast.Attribute))
                    and name not in SKIP_DEC
                    and any(
                        w in name.lower() for w in ("skip", "needs", "requires", "only", "xfail")
                    )
                ):
                    by["unresolved decorator that may skip"][(project, ast.unparse(d))] += 1
                    continue
                if name not in SKIP_DEC:
                    continue
                k = kind(name, target)
                record(
                    project,
                    rel,
                    fn.lineno,
                    f"{site} / decorator {via}",
                    k,
                    cond_kind(cond_of(target, name)),
                    reason_of(target, name),
                )
            for n in (x for st in fn.body for x in ast.walk(st)):
                if (
                    isinstance(n, ast.Call)
                    and tail(n) in BODY_CALLS
                    and ast.unparse(n.func).split(".")[0] in ("pytest", "self", "unittest")
                ):
                    c = enclosing_if(fn, n)
                    record(
                        project,
                        rel,
                        n.lineno,
                        f"{site} / body",
                        "body " + tail(n),
                        cond_kind(c) if c is not None else "unconditional",
                        reason_of(n, tail(n)),
                    )
                if (
                    isinstance(n, ast.Raise)
                    and n.exc is not None
                    and "SkipTest" in ast.unparse(n.exc)
                ):
                    c = enclosing_if(fn, n)
                    record(
                        project,
                        rel,
                        n.lineno,
                        f"{site} / body",
                        "raise SkipTest",
                        cond_kind(c) if c is not None else "unconditional",
                        "n/a",
                    )

print("tests:", dict(totals))
for section, cnt in by.items():
    print(f"\n## {section}")
    for k, v in sorted(cnt.items(), key=lambda kv: (-kv[1], str(kv[0]))):
        print(f"{v:5d}  {' | '.join(map(str, k)) if isinstance(k, tuple) else k}")
print("\n## examples")
for k, v in sorted(examples.items(), key=str):
    print(" | ".join(k), "->", ", ".join(map(str, v)))
