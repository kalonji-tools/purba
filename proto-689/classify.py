"""PROTOTYPE (#689): judge each module-level `def test*` as isolated from its TEXT alone.

Three levels, each stricter than the one before:
  L0 signature   no parameter names an input that outlives the test
  L1 + body      the body shows no write to state outside the test
  L2 + calls     the body calls nothing but builtins and its own locals
Prints one JSON object: {"<file>::<func>": {...}}.
"""

import ast, json, sys, pathlib, builtins

TESTS = pathlib.Path(sys.argv[1])

MUTATING_METHODS = {
    "append",
    "extend",
    "insert",
    "pop",
    "remove",
    "clear",
    "update",
    "setdefault",
    "add",
    "discard",
    "popitem",
    "sort",
    "reverse",
    "__setitem__",
    "__delitem__",
}
GLOBAL_MUTATORS = {
    ("os", "chdir"),
    ("os", "putenv"),
    ("os", "unsetenv"),
    ("os", "umask"),
    ("sys", "setrecursionlimit"),
    ("sys", "settrace"),
    ("sys", "setprofile"),
    ("signal", "signal"),
    ("locale", "setlocale"),
    ("logging", "basicConfig"),
    ("logging", "disable"),
    ("warnings", "simplefilter"),
    ("warnings", "filterwarnings"),
    ("random", "seed"),
    ("socket", "setdefaulttimeout"),
    ("time", "tzset"),
    ("builtins", "setattr"),
    ("builtins", "delattr"),
}
SHARED_ROOTS = {
    "os.environ",
    "sys.path",
    "sys.modules",
    "sys.argv",
    "sys.stdout",
    "sys.stderr",
    "sys.stdin",
}
BUILTIN_NAMES = set(dir(builtins))
BUILTIN_FIXTURES_FUNCTION = {
    "monkeypatch",
    "tmp_path",
    "tmpdir",
    "capsys",
    "capfd",
    "capsysbinary",
    "capfdbinary",
    "caplog",
    "recwarn",
    "request",
    "pytestconfig",
}
BUILTIN_FIXTURES_WIDER = {
    "tmp_path_factory": "session",
    "tmpdir_factory": "session",
    "pytestconfig": "session",
}


def dotted(node):
    parts = []
    while isinstance(node, ast.Attribute):
        parts.append(node.attr)
        node = node.value
    if isinstance(node, ast.Name):
        parts.append(node.id)
        return ".".join(reversed(parts))
    if isinstance(node, (ast.Subscript, ast.Call)):
        inner = dotted(node.value if isinstance(node, ast.Subscript) else node.func)
        return inner
    return None


def root(node):
    while isinstance(node, (ast.Attribute, ast.Subscript, ast.Call, ast.Starred)):
        node = node.func if isinstance(node, ast.Call) else node.value
    return node.id if isinstance(node, ast.Name) else None


def fixture_scopes(paths):
    """name -> scope, from every @pytest.fixture in conftest and test files."""
    scopes = {}
    for path in paths:
        for node in ast.walk(ast.parse(path.read_text(), str(path))):
            if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                continue
            for deco in node.decorator_list:
                call = deco if isinstance(deco, ast.Call) else None
                name = dotted(call.func if call else deco) or ""
                if name.endswith("fixture"):
                    scope = "function"
                    autouse = False
                    for kw in call.keywords if call else []:
                        if kw.arg == "scope" and isinstance(kw.value, ast.Constant):
                            scope = kw.value.value
                        if kw.arg == "autouse" and isinstance(kw.value, ast.Constant):
                            autouse = bool(kw.value.value)
                    scopes[node.name] = {"scope": scope, "autouse": autouse}
    return scopes


def parametrized(func):
    names = set()
    for deco in func.decorator_list:
        if isinstance(deco, ast.Call) and (dotted(deco.func) or "").endswith("parametrize"):
            arg = deco.args[0] if deco.args else None
            if isinstance(arg, ast.Constant) and isinstance(arg.value, str):
                names |= {n.strip() for n in arg.value.split(",")}
            elif isinstance(arg, (ast.List, ast.Tuple)):
                names |= {e.value for e in arg.elts if isinstance(e, ast.Constant)}
    return names


def locals_of(func):
    names = {a.arg for a in ast.walk(func) if isinstance(a, ast.arg)}
    for node in ast.walk(func):
        if isinstance(node, ast.Name) and isinstance(node.ctx, ast.Store):
            names.add(node.id)
        elif (
            isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef))
            and node is not func
        ):
            names.add(node.name)
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            names |= {(a.asname or a.name).split(".")[0] for a in node.names}
        elif isinstance(node, ast.ExceptHandler) and node.name:
            names.add(node.name)
        elif isinstance(node, (ast.MatchAs, ast.MatchStar)) and node.name:
            names.add(node.name)
    return names


def body_writes(func, local):
    """Each visible write to state the test does not own, as a short reason."""
    found = []
    for node in ast.walk(func):
        if isinstance(node, ast.Global) or (
            isinstance(node, ast.Nonlocal) and not set(node.names) <= local
        ):
            found.append(f"{type(node).__name__.lower()} {', '.join(node.names)}")
        targets = []
        if isinstance(node, ast.Assign):
            targets = node.targets
        elif isinstance(node, (ast.AugAssign, ast.AnnAssign)):
            targets = [node.target]
        elif isinstance(node, ast.Delete):
            targets = node.targets
        for t in targets:
            if isinstance(t, (ast.Attribute, ast.Subscript)):
                r = root(t)
                if r and r not in local:
                    found.append(f"writes {dotted(t) or r}")
                d = dotted(t) or ""
                if any(d == s or d.startswith(s + ".") for s in SHARED_ROOTS):
                    found.append(f"writes {d}")
        if isinstance(node, ast.Call):
            f = node.func
            name = dotted(f) or ""
            if isinstance(f, ast.Attribute) and f.attr in MUTATING_METHODS:
                r = root(f.value)
                base = dotted(f.value) or ""
                if (r and r not in local) or base in SHARED_ROOTS:
                    found.append(f"calls {name}")
            parts = tuple(name.split("."))
            if parts[-2:] in GLOBAL_MUTATORS or (
                len(parts) == 1 and ("builtins", parts[0]) in GLOBAL_MUTATORS
            ):
                if parts[0] not in local or parts[0] in {"os", "sys"}:
                    found.append(f"calls {name}")
    return sorted(set(found))


def outside_calls(func, local):
    calls = set()
    for node in ast.walk(func):
        if isinstance(node, ast.Call):
            r = root(node.func)
            if r is None or (r not in local and r not in BUILTIN_NAMES):
                calls.add(dotted(node.func) or "<expr>")
            elif r in local and r in {a.arg for a in func.args.args}:
                calls.add(dotted(node.func) or "<expr>")  # a method of an input runs project code
    return sorted(calls)


def main():
    files = sorted(TESTS.glob("test_*.py"))
    scopes = fixture_scopes(files + sorted(TESTS.glob("conftest.py")))
    autouse = sorted(n for n, s in scopes.items() if s["autouse"])
    out = {}
    for path in files:
        tree = ast.parse(path.read_text(), str(path))
        for func in tree.body:
            if not isinstance(
                func, (ast.FunctionDef, ast.AsyncFunctionDef)
            ) or not func.name.startswith("test"):
                continue
            params = [a.arg for a in func.args.posonlyargs + func.args.args + func.args.kwonlyargs]
            values = parametrized(func)
            wider = []
            for p in params:
                if p in values:
                    continue
                scope = scopes.get(p, {}).get("scope") or BUILTIN_FIXTURES_WIDER.get(p)
                if p not in BUILTIN_FIXTURES_FUNCTION and scope not in (None, "function"):
                    wider.append(f"{p} lives for the {scope}")
            local = locals_of(func)
            writes = body_writes(func, local)
            calls = outside_calls(func, local)
            out[f"{path.name}::{func.name}"] = {
                "params": params,
                "L0_signature": not wider and not autouse,
                "L1_body": not wider and not autouse and not writes,
                "L2_calls": not wider and not autouse and not writes and not calls,
                "wider_inputs": wider,
                "writes": writes,
                "outside_calls": len(calls),
            }
    json.dump({"autouse": autouse, "tests": out}, sys.stdout, indent=1)


main()
