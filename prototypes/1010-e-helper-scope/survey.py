"""Where do the helpers that a test calls before its first assert live?
Prototype of #1010, Q4. Never merge.

    python -I survey.py <project root> <name>     prints a summary, writes runs/<name>.json

It reads the project's files with `ast` only: it never imports or runs them. For each test
function it takes the act (every call that sits before the test's first check) and resolves each
call to where its callee is defined:

  a  the test's own module
  b  another module of test code (conftest, helpers, utils, a base TestCase)
  c  code under test that sits in the test file's folder or below it
     (what a folder scope would also watch)
  d  code under test outside that folder, the standard library, a third party, or a builtin

For each callee of kind a or b it records whether its body branches. A test's first check is
its first `assert`
statement, or its first `self.assert…` / `self.fail…` call (unittest style), in source order.
"""

from __future__ import annotations

import ast
import builtins
import json
import os
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass, field

SKIP_DIRS = {
    ".git",
    ".tox",
    ".nox",
    ".venv",
    "venv",
    "build",
    "dist",
    "node_modules",
    "docs",
    "doc",
    "documentation",
    "bench",
    "benchmarks",
    "typing_tests",
    "__pycache__",
}
TEST_DIRS = {"test", "tests", "testing"}
NOT_UNDER_TEST = {"setup.py", "noxfile.py", "conftest.py", "tasks.py", "fabfile.py"}
BUILTINS = set(dir(builtins))
CHECK_METHODS = ("assert", "fail")
# A branch as purba's one-path rule reads it. `for` is kept apart: #980's form lets a loop over
# any data pass, as
# long as it runs a round, so a `for` is a weaker branch than an `if`.
STRICT = ("if", "ifexp", "boolop", "while", "except", "match", "comp-if")
LOOPS = ("for", "comp-for")


@dataclass
class Module:
    path: str
    tree: ast.Module | None
    defs: dict[str, ast.AST] = field(default_factory=dict)  # name -> FunctionDef / ClassDef
    imports: dict[str, tuple[str, str | None, int]] = field(
        default_factory=dict
    )  # name -> (module, attr, level)
    stars: list[tuple[str, int]] = field(default_factory=list)
    values: set[str] = field(default_factory=set)


class Project:
    def __init__(self, root: str) -> None:
        self.root = os.path.realpath(root)
        self.mods: dict[str, Module] = {}
        self.files: list[str] = []
        for dirpath, dirnames, filenames in os.walk(self.root):
            dirnames[:] = sorted(
                d for d in dirnames if d not in SKIP_DIRS and not d.startswith(".")
            )
            self.files += [os.path.join(dirpath, f) for f in sorted(filenames) if f.endswith(".py")]
        self.syntax_errors: list[str] = []

    def rel(self, path: str) -> str:
        return os.path.relpath(path, self.root)

    def is_test_code(self, path: str) -> bool:
        parts = self.rel(path).split(os.sep)
        base = parts[-1]
        return (
            base.startswith("test_")
            or base.endswith("_test.py")
            or base == "conftest.py"
            or any(p in TEST_DIRS for p in parts[:-1])
        )

    def is_test_file(self, path: str) -> bool:
        base = os.path.basename(path)
        return base.startswith("test_") or base.endswith("_test.py")

    def module(self, path: str) -> Module:
        if path not in self.mods:
            try:
                with open(path, encoding="utf-8", errors="replace") as f:
                    tree = ast.parse(f.read(), path)
            except SyntaxError:
                self.syntax_errors.append(self.rel(path))
                tree = None
            mod = self.mods[path] = Module(path, tree)
            if tree is not None:
                self.index(mod, tree.body)
        return self.mods[path]

    def index(self, mod: Module, body: list[ast.stmt]) -> None:
        """Module-level names, including those bound under a top-level `if` or `try`
        (six binds most of its names so)."""
        for s in body:
            if isinstance(s, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                mod.defs[s.name] = s
                mod.imports.pop(s.name, None)
            elif isinstance(s, ast.Import):
                for a in s.names:
                    if a.asname:
                        mod.imports[a.asname] = (a.name, None, 0)
                    else:
                        top = a.name.split(".")[0]
                        mod.imports[top] = (top, None, 0)
            elif isinstance(s, ast.ImportFrom):
                for a in s.names:
                    if a.name == "*":
                        mod.stars.append((s.module or "", s.level))
                    else:
                        mod.imports[a.asname or a.name] = (s.module or "", a.name, s.level)
            elif isinstance(s, (ast.Assign, ast.AnnAssign)):
                targets = s.targets if isinstance(s, ast.Assign) else [s.target]
                for t in targets:
                    for n in ast.walk(t):
                        if isinstance(n, ast.Name):
                            mod.values.add(n.id)
            elif isinstance(s, (ast.If, ast.Try)):
                self.index(
                    mod,
                    s.body
                    + s.orelse
                    + getattr(s, "finalbody", [])
                    + [x for h in getattr(s, "handlers", []) for x in h.body],
                )

    def find_module(self, name: str, level: int, from_file: str) -> str | None:
        """A file of this project that `import name` would load from `from_file`,
        or None (stdlib, third party)."""
        parts = [p for p in name.split(".") if p]
        if level:
            base = os.path.dirname(from_file)
            for _ in range(level - 1):
                base = os.path.dirname(base)
            roots = [base]
        else:
            roots, d = [], os.path.dirname(from_file)
            while d.startswith(
                self.root
            ):  # pytest's rootdir-based sys.path entries: the test's folder and up
                roots.append(d)
                if d == self.root:
                    break
                d = os.path.dirname(d)
            roots.append(os.path.join(self.root, "src"))
        for r in roots:
            p = os.path.join(r, *parts)
            for cand in (p + ".py", os.path.join(p, "__init__.py")):
                if parts and os.path.isfile(cand):
                    return cand
            if not parts and os.path.isfile(os.path.join(r, "__init__.py")):
                return os.path.join(r, "__init__.py")
        return None

    def lookup(
        self, path: str, name: str, depth: int = 0
    ) -> tuple[str, str | None, ast.AST | None]:
        """Where `name` of module `path` is defined: ("def", file, node), ("module", file, None),
        ("value", file, None), ("external", None, None) or ("unknown", file, None)."""
        if depth > 8:
            return ("unknown", path, None)
        mod = self.module(path)
        if name in mod.defs:
            return ("def", path, mod.defs[name])
        if name in mod.imports:
            m, attr, level = mod.imports[name]
            if attr is None:
                f = self.find_module(m, level, path)
                return ("module", f, None) if f else ("external", None, None)
            f = self.find_module(m, level, path)
            if f is None:
                return ("external", None, None)
            if os.path.basename(f) == "__init__.py":
                sub = self.find_module(attr, 1, f)
                if sub and attr not in self.module(f).defs and attr not in self.module(f).imports:
                    return ("module", sub, None)
            return self.lookup(f, attr, depth + 1)
        if name in mod.values:
            return ("value", path, None)
        for m, level in mod.stars:
            f = self.find_module(m, level, path)
            if f:
                got = self.lookup(f, name, depth + 1)
                if got[0] != "unknown":
                    return got
        if os.path.basename(path) == "__init__.py":
            sub = self.find_module(name, 1, path)
            if sub:
                return ("module", sub, None)
        return ("unknown", path, None)


def branches(fn: ast.AST) -> Counter[str]:
    """The branches in a function's own body.
    A nested def or lambda is not its body: it may be a callback."""
    out: Counter[str] = Counter()

    def literal(it: ast.expr) -> bool:
        if isinstance(it, (ast.List, ast.Tuple, ast.Set)):
            return all(isinstance(e, ast.Constant) for e in it.elts)
        return (
            isinstance(it, ast.Call)
            and isinstance(it.func, ast.Name)
            and it.func.id == "range"
            and all(isinstance(a, ast.Constant) for a in it.args)
        )

    def walk(node: ast.AST) -> None:
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda, ast.ClassDef)):
                continue
            if isinstance(child, ast.If):
                out["if"] += 1
            elif isinstance(child, ast.IfExp):
                out["ifexp"] += 1
            elif isinstance(child, ast.BoolOp):
                out["boolop"] += 1
            elif isinstance(child, ast.While):
                out["while"] += 1
            elif isinstance(child, ast.Try) and child.handlers:
                out["except"] += 1
            elif isinstance(child, ast.Match):
                out["match"] += 1
            elif isinstance(child, (ast.For, ast.AsyncFor)) and not literal(child.iter):
                out["for"] += 1
            elif isinstance(child, ast.comprehension):
                if not literal(child.iter):
                    out["comp-for"] += 1
                out["comp-if"] += len(child.ifs)
            walk(child)

    walk(fn)
    return out


def body_of(node: ast.AST) -> list[ast.AST]:
    """The code a call of this def runs: a function's body, or a class's `__init__`,
    `__post_init__`, `__new__`, and
    `__enter__`, `__exit__`, `__call__`, which a `with` or a call of the instance runs next."""
    if isinstance(node, ast.ClassDef):
        return [
            s
            for s in node.body
            if isinstance(s, (ast.FunctionDef, ast.AsyncFunctionDef))
            and s.name
            in ("__init__", "__post_init__", "__new__", "__enter__", "__exit__", "__call__")
        ]
    return [node]


def driven(node: ast.AST) -> bool:
    """A generator that a context manager or pytest drives. Its frames are called by contextlib
    or pytest, not by the test, so purba's watch reads it as a value whose paths are free, in the
    test's module too (confirm/escape/)."""
    return any(
        w in ast.unparse(d)
        for d in getattr(node, "decorator_list", [])
        for w in ("contextmanager", "contextdecorator", "fixture")
    )


def is_check(n: ast.AST) -> bool:
    """An `assert`, or a unittest check `self.assert…(…)` / `self.fail…(…)`."""
    return isinstance(n, ast.Assert) or (
        isinstance(n, ast.Call)
        and isinstance(n.func, ast.Attribute)
        and isinstance(n.func.value, ast.Name)
        and n.func.value.id == "self"
        and n.func.attr.startswith(CHECK_METHODS)
    )


def first_check(fn: ast.AST) -> int | None:
    """The line of a test's first check, in source order, outside nested defs."""
    best: list[int] = []

    def walk(node: ast.AST) -> None:
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda, ast.ClassDef)):
                continue
            if is_check(child):
                best.append(child.lineno)
            walk(child)

    walk(fn)
    return min(best) if best else None


def calls_in(fn: ast.AST, before: int | None) -> list[ast.Call]:
    out: list[ast.Call] = []

    def walk(node: ast.AST) -> None:
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda, ast.ClassDef)):
                continue
            if isinstance(child, ast.Call) and (before is None or child.lineno < before):
                out.append(child)
            walk(child)

    for s in getattr(fn, "body", []):
        if isinstance(s, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            continue
        if before is not None and s.lineno >= before and not hasattr(s, "body"):
            continue
        if isinstance(s, ast.Call) and (before is None or s.lineno < before):
            out.append(s)
        walk(s)
    return out


class Survey:
    def __init__(self, proj: Project) -> None:
        self.p = proj
        self.trans: dict[int, bool] = {}

    def local_names(self, fn: ast.AST) -> tuple[set[str], set[str]]:
        params = (
            {a.arg for a in ast.walk(fn.args) if isinstance(a, ast.arg)}
            if hasattr(fn, "args")
            else set()
        )
        bound: set[str] = set()
        for n in ast.walk(fn):
            if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Store):
                bound.add(n.id)
            elif isinstance(n, (ast.withitem,)) and n.optional_vars is not None:
                bound |= {x.id for x in ast.walk(n.optional_vars) if isinstance(x, ast.Name)}
            elif isinstance(n, ast.ExceptHandler) and n.name:
                bound.add(n.name)
            elif isinstance(n, (ast.Import, ast.ImportFrom)):
                bound |= {(a.asname or a.name).split(".")[0] for a in n.names}
        return params, bound

    def nested_def(self, fn: ast.AST, name: str) -> ast.AST | None:
        for n in ast.walk(fn):
            if (
                n is not fn
                and isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef))
                and n.name == name
            ):
                return n
        return None

    def fixture(self, test_file: str, name: str) -> tuple[str, ast.AST] | None:
        """The pytest fixture a parameter names: in the test module, then in each conftest.py
        from its folder up."""
        cands, d = [test_file], os.path.dirname(test_file)
        while d.startswith(self.p.root):
            cands.append(os.path.join(d, "conftest.py"))
            if d == self.p.root:
                break
            d = os.path.dirname(d)
        for f in cands:
            if not os.path.isfile(f):
                continue
            got = self.p.lookup(f, name)
            if (
                got[0] == "def"
                and isinstance(got[2], (ast.FunctionDef, ast.AsyncFunctionDef))
                and any("fixture" in ast.unparse(dec) for dec in got[2].decorator_list)
            ):
                return got[1], got[2]
        return None

    def made_by(self, path: str, fn: ast.AST, scope: ast.AST, name: str | None = None):
        """What a fixture returns or yields (name None), or what a local `name = Call(…)` holds,
        when it is an instance of a class or a function this project defines:
        ("instance", file, ClassDef) or ("def", file, node)."""
        values: list[ast.expr] = []
        for n in ast.walk(fn):
            if name is None and isinstance(n, (ast.Return, ast.Yield)) and n.value is not None:
                values.append(n.value)
            elif (
                name is not None
                and isinstance(n, ast.Assign)
                and any(isinstance(t, ast.Name) and t.id == name for t in n.targets)
            ):
                values.append(n.value)
        if len(values) != 1:
            return None
        v = values[0]
        cls = self.enclosing_class(path, fn) if name is None else None
        if isinstance(v, ast.Call):
            got = self.resolve_expr(path, v.func, scope if name is None else None, cls)
            if got[0] == "def" and isinstance(got[2], ast.ClassDef):
                return ("instance", got[1], got[2])
        elif isinstance(v, ast.Name) and name is None:
            inner = self.nested_def(fn, v.id)
            if inner is not None:
                return ("def", path, inner)
            got = self.p.lookup(path, v.id)
            if got[0] == "def":
                return ("instance", got[1], got[2]) if False else ("def", got[1], got[2])
            if v.id != name:
                return self.made_by(path, fn, scope, v.id)
        return None

    def method(
        self, path: str, cls: ast.ClassDef, attr: str, depth: int = 0
    ) -> tuple[str, str | None, ast.AST | None]:
        """A method found through a class's bases, as `self.attr` would find it."""
        if depth > 8:
            return ("unknown", path, None)
        for s in cls.body:
            if isinstance(s, (ast.FunctionDef, ast.AsyncFunctionDef)) and s.name == attr:
                return ("def", path, s)
            if isinstance(s, (ast.Assign, ast.AnnAssign)):
                tg = s.targets if isinstance(s, ast.Assign) else [s.target]
                if any(isinstance(t, ast.Name) and t.id == attr for t in tg):
                    return ("value", path, None)
        for b in cls.bases:
            got = self.resolve_expr(path, b, None, None)
            if got[0] == "def" and isinstance(got[2], ast.ClassDef):
                found = self.method(got[1], got[2], attr, depth + 1)
                if found[0] != "unknown":
                    return found
            elif got[0] == "external":
                return ("external", None, None)  # unittest.TestCase and the like
        return ("unknown", path, None)

    def resolve_expr(self, path: str, expr: ast.expr, fn: ast.AST | None, cls: ast.ClassDef | None):
        """Resolve a callee expression. Returns (kind, file, node, why).
        `why` names an unresolved shape."""
        if isinstance(expr, ast.Name):
            if fn is not None:
                params, bound = self.local_names(fn)
                nested = self.nested_def(fn, expr.id)
                if nested is not None:
                    return ("def", path, nested, None)
                if expr.id in params:
                    fx = self.fixture(path, expr.id)
                    if fx is not None:
                        made = self.made_by(fx[0], fx[1], fx[1])
                        if made is not None:
                            return (*made, None)
                    return ("unresolved", None, None, "a parameter (fixture or input)")
                if expr.id in bound:
                    made = self.made_by(path, fn, fn, expr.id)
                    if made is not None:
                        return (*made, None)
                    return ("unresolved", None, None, "a local name")
            got = self.p.lookup(path, expr.id)
            if got[0] == "unknown" and expr.id in BUILTINS:
                return ("builtin", None, None, None)
            return (*got, None)
        if isinstance(expr, ast.Attribute):
            v = expr.value
            if isinstance(v, ast.Name) and v.id in ("self", "cls") and cls is not None:
                got = self.method(path, cls, expr.attr)
                return (
                    (*got, None)
                    if got[0] != "unknown"
                    else ("unresolved", None, None, "an attribute of self")
                )
            if isinstance(v, (ast.Name, ast.Attribute)):
                base = self.resolve_expr(path, v, fn, cls)
                if base[0] == "module" and base[1]:
                    got = self.p.lookup(base[1], expr.attr)
                    if got[0] == "unknown" and os.path.basename(base[1]) == "__init__.py":
                        sub = self.p.find_module(expr.attr, 1, base[1])
                        if sub:
                            return ("module", sub, None, None)
                    return (*got, None)
                if base[0] == "external":
                    return ("external", None, None, None)
                if base[0] == "def" and isinstance(base[2], ast.ClassDef):
                    got = self.method(base[1], base[2], expr.attr)
                    return (
                        (*got, None)
                        if got[0] != "unknown"
                        else ("unresolved", None, None, "a method of a class")
                    )
                if base[0] == "builtin":
                    return ("builtin", None, None, None)
                if base[0] == "instance":
                    got = self.method(base[1], base[2], expr.attr)
                    if got[0] != "unknown":
                        return (*got, None)
                    return ("method-of-value", base[1], None, None)
                if base[0] == "value" and base[1]:
                    # A method of a module-level object, such as loguru's `logger`:
                    # it lives where its object does.
                    return ("method-of-value", base[1], None, None)
                return ("unresolved", None, None, "a method of a value")
            return ("unresolved", None, None, "a method of a value")
        return ("unresolved", None, None, "a computed callee")

    def kind(self, test_file: str, got) -> str:
        k, f = got[0], got[1]
        if k == "unresolved":
            return "unresolved"
        if k in ("external", "builtin"):
            return "d"
        if k == "method-of-value":
            return (
                "a"
                if f == test_file
                else "b"
                if self.p.is_test_code(f)
                else "c"
                if f.startswith(os.path.dirname(test_file) + os.sep)
                else "d"
            )
        if f is None or k == "unknown" and f == test_file:
            return "unresolved" if k == "unknown" else "d"
        if k == "unknown":
            return "unresolved"
        if f == test_file:
            return "a"
        if self.p.is_test_code(f):
            return "b"
        if f.startswith(os.path.dirname(test_file) + os.sep):
            return "c"
        return "d"

    def enclosing_class(self, path: str, node: ast.AST) -> ast.ClassDef | None:
        tree = self.p.module(path).tree
        for c in ast.walk(tree) if tree else []:
            if isinstance(c, ast.ClassDef) and node in c.body:
                return c
        return None

    def branchy(
        self, path: str, node: ast.AST, seen: frozenset[int] = frozenset()
    ) -> tuple[Counter[str], bool]:
        """Own branches, and whether it or test code it calls branches
        (its own module or another of test code)."""
        own: Counter[str] = Counter()
        for b in body_of(node):
            own += branches(b)
        if id(node) in self.trans:
            return own, self.trans[id(node)]
        if any(own[k] for k in STRICT) or id(node) in seen:
            self.trans[id(node)] = any(own[k] for k in STRICT)
            return own, self.trans[id(node)]
        result = False
        cls = self.enclosing_class(path, node)
        for b in body_of(node):
            for call in calls_in(b, None):
                got = self.resolve_expr(
                    path,
                    call.func,
                    b,
                    cls if b is node else (node if isinstance(node, ast.ClassDef) else cls),
                )
                if got[0] == "def" and got[1] and self.p.is_test_code(got[1]):
                    if self.branchy(got[1], got[2], seen | {id(node)})[1]:
                        result = True
                        break
            if result:
                break
        self.trans[id(node)] = result
        return own, result


def tests_of(tree: ast.Module):
    for s in tree.body:
        if isinstance(s, (ast.FunctionDef, ast.AsyncFunctionDef)) and s.name.startswith("test"):
            yield s, None
        elif isinstance(s, ast.ClassDef) and (s.name.startswith("Test") or s.bases):
            for m in s.body:
                if isinstance(m, (ast.FunctionDef, ast.AsyncFunctionDef)) and m.name.startswith(
                    "test"
                ):
                    yield m, s


def run(root: str, name: str, out_dir: str) -> dict:
    p = Project(root)
    sv = Survey(p)
    tests: list[dict] = []
    helpers: dict[tuple[str, str, int], dict] = {}
    unresolved: Counter[str] = Counter()
    samples: dict[str, list[str]] = defaultdict(list)
    calls_by_kind: Counter[str] = Counter()
    test_files = [f for f in p.files if p.is_test_file(f)]
    for tf in test_files:
        mod = p.module(tf)
        if mod.tree is None:
            continue
        for fn, cls in tests_of(mod.tree):
            limit = first_check(fn)
            rec = {
                "file": p.rel(tf),
                # A location, not a name: a spelling gate would rewrite a project's own names.
                "test": f"{p.rel(tf)}:{fn.lineno}",
                "has_check": limit is not None,
                "calls": Counter(),
                "b": [],
                "a": [],
                "c": [],
                "fixtures": [],
            }
            for call in calls_in(fn, limit):
                got = sv.resolve_expr(tf, call.func, fn, cls)
                k = sv.kind(tf, got)
                rec["calls"][k] += 1
                calls_by_kind[k] += 1
                if k == "unresolved":
                    shape = got[3] or "a name bound by no def"
                    unresolved[shape] += 1
                    if len(samples[shape]) < 12:
                        samples[shape].append(f"{p.rel(tf)}:{call.lineno}")
                    continue
                if k in ("a", "b", "c") and got[0] == "def":
                    node = got[2]
                    key = (p.rel(got[1]), getattr(node, "name", "?"), node.lineno)
                    if key not in helpers:
                        own, trans = sv.branchy(got[1], node)
                        helpers[key] = {
                            "file": key[0],
                            "name": key[1],
                            "line": key[2],
                            "kind": k,
                            "class": isinstance(node, ast.ClassDef),
                            "own": dict(own),
                            "strict": any(own[x] for x in STRICT),
                            "loop": any(own[x] for x in LOOPS),
                            "transitive": trans,
                            "driven": driven(node),
                            "asserts": any(is_check(n) for b in body_of(node) for n in ast.walk(b)),
                            "in_folder": got[1].startswith(os.path.dirname(tf) + os.sep),
                        }
                    if key not in rec[k]:
                        rec[k].append(key)
            for param in {a.arg for a in fn.args.args + fn.args.kwonlyargs}:
                fx = sv.fixture(tf, param)
                if fx and fx[0] != tf and p.is_test_code(fx[0]):
                    rec["fixtures"].append(
                        [p.rel(fx[0]), param, any(branches(fx[1])[x] for x in STRICT)]
                    )
            rec["calls"] = dict(rec["calls"])
            tests.append(rec)

    # The folder scope's false-positive surface:
    # code under test in a test file's folder or below it.
    surface: dict[str, dict] = {}
    for d in sorted({os.path.dirname(f) for f in test_files}):
        mods = [
            f
            for f in p.files
            if f.startswith(d + os.sep)
            and not p.is_test_code(f)
            and os.path.basename(f) not in NOT_UNDER_TEST
        ]
        fns = branchy_fns = 0
        for f in mods:
            t = p.module(f).tree
            for n in ast.walk(t) if t else []:
                if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    fns += 1
                    branchy_fns += any(branches(n)[x] for x in STRICT)
        if mods:
            surface[p.rel(d) or "."] = {
                "modules": len(mods),
                "functions": fns,
                "branching_functions": branchy_fns,
                "module_list": [p.rel(f) for f in mods],
            }

    def tests_where(pred) -> int:
        return sum(1 for t in tests if pred(t))

    hb = {k: v for k, v in helpers.items() if v["kind"] == "b"}
    ha = {k: v for k, v in helpers.items() if v["kind"] == "a"}
    hc = {k: v for k, v in helpers.items() if v["kind"] == "c"}
    summary = {
        "project": name,
        "test_files": len(test_files),
        "tests": len(tests),
        "tests_without_check": tests_where(lambda t: not t["has_check"]),
        "tests_calling_b": tests_where(lambda t: t["b"]),
        "b_helpers": len(hb),
        "b_helpers_strict_branch": sum(v["strict"] for v in hb.values()),
        "b_helpers_branch_or_loop": sum(v["strict"] or v["loop"] for v in hb.values()),
        "b_helpers_transitive": sum(v["transitive"] for v in hb.values()),
        "b_helpers_in_folder": sum(v["in_folder"] for v in hb.values()),
        "b_helpers_asserting": sum(v["asserts"] for v in hb.values()),
        "tests_calling_asserting_b": tests_where(
            lambda t: any(helpers[k]["asserts"] for k in t["b"])
        ),
        "b_helpers_driven": sum(v["driven"] for v in hb.values()),
        "tests_escape_arrange": tests_where(
            lambda t: any(
                helpers[k]["transitive"] and not helpers[k]["asserts"] and not helpers[k]["driven"]
                for k in t["b"]
            )
        ),
        "tests_escape_arrange_incl_driven": tests_where(
            lambda t: any(helpers[k]["transitive"] and not helpers[k]["asserts"] for k in t["b"])
        ),
        "tests_escape_assert_with_check": tests_where(
            lambda t: t["has_check"] and any(helpers[k]["asserts"] for k in t["b"])
        ),
        "tests_escape_strict": tests_where(lambda t: any(helpers[k]["strict"] for k in t["b"])),
        "tests_escape_transitive": tests_where(
            lambda t: any(helpers[k]["transitive"] for k in t["b"])
        ),
        "tests_escape_outside_folder": tests_where(
            lambda t: any(
                helpers[k]["transitive"]
                and not helpers[k]["driven"]
                and not helpers[k]["in_folder"]
                for k in t["b"]
            )
        ),
        "tests_using_b_fixture": tests_where(lambda t: t["fixtures"]),
        "tests_using_branching_b_fixture": tests_where(lambda t: any(x[2] for x in t["fixtures"])),
        "tests_calling_a": tests_where(lambda t: t["a"]),
        "tests_calling_branching_a": tests_where(
            lambda t: any(helpers[k]["transitive"] and not helpers[k]["driven"] for k in t["a"])
        ),
        "a_helpers": len(ha),
        "a_helpers_strict_branch": sum(v["strict"] for v in ha.values()),
        "tests_calling_c": tests_where(lambda t: t["c"]),
        "tests_calling_branching_c": tests_where(
            lambda t: any(helpers[k]["strict"] for k in t["c"])
        ),
        "c_callees": len(hc),
        "c_callees_strict_branch": sum(v["strict"] for v in hc.values()),
        "folder_surface": {
            k: {x: y for x, y in v.items() if x != "module_list"} for k, v in surface.items()
        },
        "calls": dict(calls_by_kind),
        "unresolved_share": round(
            calls_by_kind["unresolved"] / max(1, sum(calls_by_kind.values())), 3
        ),
        "unresolved_by_shape": dict(unresolved.most_common()),
        "syntax_errors": p.syntax_errors,
    }
    detail = {
        "summary": summary,
        "surface": surface,
        "unresolved_samples": samples,
        "helpers": sorted(helpers.values(), key=lambda v: (v["kind"], v["file"], v["line"])),
        "tests": [
            {
                **t,
                "a": [list(k) for k in t["a"]],
                "b": [list(k) for k in t["b"]],
                "c": [list(k) for k in t["c"]],
            }
            for t in tests
        ],
    }
    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, f"{name}.json"), "w") as f:
        json.dump(detail, f, indent=1, sort_keys=False)
        f.write("\n")
    return summary


ROWS = [  # (key, label) of the table that `--table` prints
    ("tests", "test functions"),
    ("tests_calling_b", "tests calling a helper of another test module (b)"),
    ("b_helpers", "– distinct b helpers they call"),
    ("b_helpers_transitive", "– of them, branching (own body, or a test helper it calls)"),
    ("b_helpers_driven", "– of them, driven by contextlib or pytest (free in any scope)"),
    ("b_helpers_asserting", "– of them, asserting"),
    (
        "tests_escape_arrange",
        "ESCAPE under module scope: tests whose act calls a branching b helper",
    ),
    ("tests_escape_arrange_incl_driven", "– counting driven helpers too"),
    (
        "tests_escape_assert_with_check",
        "tests calling an asserting b helper before their own check",
    ),
    (
        "tests_escape_outside_folder",
        "tests whose branching b helper sits outside the folder scope too",
    ),
    (
        "tests_calling_branching_a",
        "tests calling a branching helper of their own module (refused today)",
    ),
    (
        "tests_using_branching_b_fixture",
        "tests taking a branching conftest fixture (not a call; not counted)",
    ),
    ("folder_modules", "FOLDER SCOPE: modules of code under test in a test file's folder"),
    ("folder_functions", "– their functions"),
    ("folder_branching_functions", "– of them, branching"),
    ("tests_calling_c", "tests whose act calls that code, resolved statically"),
    ("tests_calling_branching_c", "– calling a branching function of it"),
    ("unresolved_share", "calls in acts that the analyser could not resolve"),
]


def table(runs: str, names: list[str]) -> str:
    sums = {}
    for n in names:
        with open(os.path.join(runs, f"{n}.json")) as f:
            s = json.load(f)["summary"]
        surf = s["folder_surface"].values()
        s["folder_modules"] = sum(v["modules"] for v in surf)
        s["folder_functions"] = sum(v["functions"] for v in surf)
        s["folder_branching_functions"] = sum(v["branching_functions"] for v in surf)
        sums[n] = s
    out = ["| | " + " | ".join(names) + " | all |", "|---" * (len(names) + 2) + "|"]
    for key, label in ROWS:
        vals = [sums[n][key] for n in names]
        total = f"{sum(vals) / len(vals):.3f}" if key == "unresolved_share" else str(sum(vals))
        out.append(f"| {label.strip()} | " + " | ".join(map(str, vals)) + f" | {total} |")
    return "\n".join(out)


if __name__ == "__main__":
    here = os.path.dirname(os.path.abspath(__file__))
    if sys.argv[1] == "--table":
        print(table(os.path.join(here, "runs"), sys.argv[2:]))
    else:
        s = run(sys.argv[1], sys.argv[2], os.path.join(here, "runs"))
        print(json.dumps(s, indent=1))
