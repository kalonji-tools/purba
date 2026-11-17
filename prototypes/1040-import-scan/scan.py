"""PROTOTYPE, throwaway: never merges.

Read, with Python's ast, each test file of the 9 pinned suites, and count what
each option of #1040's questions Q3, Q4, Q6 and Q7 would refuse.

A test file is test_*.py or *_test.py. A test is a def or async def whose name
starts with `test` in the module's own body. The target is `pytest` or
`_pytest`, or a module under either.

Usage: python -I scan.py <folder of the 9 clones>
"""

import ast
import pathlib
import re
import sys
from collections import Counter, defaultdict

TARGET = {"pytest", "_pytest"}

# The calls that import a module named by their first argument, by the
# qualified name an import gives them. find_spec imports only the parent of a
# dotted name, so it counts only for a dotted name.
QUALIFIED = {
    "__import__": "__import__",
    "importlib.__import__": "__import__",
    "importlib.import_module": "import_module",
    "importlib.util.find_spec": "find_spec",
    "pkgutil.resolve_name": "resolve_name",
    "runpy.run_module": "run_module",
    "importlib.resources.files": "files",
    "unittest.mock.patch": "patch",
    "unittest.mock.patch.object": None,
}
SHORT = {"__import__", "import_module", "find_spec", "resolve_name", "run_module", "files", "patch"}


def top(name):
    return re.split(r"[.:]", name)[0]


def names_target(value, short):
    if not isinstance(value, str) or top(value) not in TARGET:
        return False
    return not (short == "find_spec" and "." not in value)


def is_type_checking(test):
    return (isinstance(test, ast.Name) and test.id == "TYPE_CHECKING") or (
        isinstance(test, ast.Attribute) and test.attr == "TYPE_CHECKING"
    )


class Walk(ast.NodeVisitor):
    """Each import of the target and each known call, with where it sits:
    module, type_checking, test, helper or class."""

    def __init__(self):
        self.where = ["module"]
        self.owner = None
        self.owned = defaultdict(set)  # test name -> lines of imports in its body
        self.imports = []  # (line, where, module)
        self.calls = []  # (line, where, short, qualified or None, value)
        self.bound = {}  # local name -> qualified origin, from every import

    def visit_If(self, node):
        if is_type_checking(node.test):
            self.where.append("type_checking")
            for child in node.body:
                self.visit(child)
            self.where.pop()
            for child in node.orelse:
                self.visit(child)
        else:
            self.generic_visit(node)

    def _def(self, node):
        parent = self.where[-1]
        if parent == "module":
            kind = "test" if node.name.startswith("test") else "helper"
            if kind == "test":
                self.owner = node.name
        elif parent in ("test", "helper", "class"):
            kind = parent
        else:
            kind = "helper"
        self.where.append(kind)
        self.generic_visit(node)
        self.where.pop()
        if parent == "module":
            self.owner = None

    visit_FunctionDef = _def
    visit_AsyncFunctionDef = _def

    def visit_ClassDef(self, node):
        self.where.append("class")
        self.generic_visit(node)
        self.where.pop()

    def visit_Import(self, node):
        for alias in node.names:
            self.bound[alias.asname or alias.name.split(".")[0]] = (
                alias.name if alias.asname else alias.name.split(".")[0]
            )
            if top(alias.name) in TARGET:
                self.imports.append((node.lineno, self.where[-1], alias.name))
                if self.owner: self.owned[self.owner].add(node.lineno)

    def visit_ImportFrom(self, node):
        if node.level:
            return
        for alias in node.names:
            self.bound[alias.asname or alias.name] = f"{node.module}.{alias.name}"
        if top(node.module) in TARGET:
            self.imports.append((node.lineno, self.where[-1], node.module))
            if self.owner: self.owned[self.owner].add(node.lineno)

    def visit_Call(self, node):
        self.generic_visit(node)
        func = node.func
        short = func.id if isinstance(func, ast.Name) else getattr(func, "attr", None)
        if short not in SHORT or not node.args:
            return
        first = node.args[0]
        value = first.value if isinstance(first, ast.Constant) else None
        self.calls.append((node.lineno, self.where[-1], short, dotted(func), value, self.owner))


def dotted(func):
    parts = []
    while isinstance(func, ast.Attribute):
        parts.append(func.attr)
        func = func.value
    if not isinstance(func, ast.Name):
        return None
    parts.append(func.id)
    return ".".join(reversed(parts))


def qualify(walk, written):
    if written is None:
        return None
    head, _, rest = written.partition(".")
    origin = walk.bound.get(head)
    if origin is None:
        return written if written == "__import__" else None
    return f"{origin}.{rest}" if rest else origin


def main(root):
    files = sorted(
        p
        for p in pathlib.Path(root).rglob("*.py")
        if p.name.startswith("test_") or p.name.endswith("_test.py")
    )
    totals = Counter()
    lines_per_file = Counter()
    by_where = Counter()
    tests_by_reach = Counter()
    helper_only = []
    tc_only = []
    call_rows = []
    for path in files:
        tree = ast.parse(path.read_text(encoding="utf-8"), str(path))
        walk = Walk()
        walk.visit(tree)
        tests = [
            n.name
            for n in tree.body
            if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name.startswith("test")
        ]
        totals["files"] += 1
        totals["tests"] += len(tests)
        lines = {line for line, *_ in walk.imports}
        if walk.imports:
            totals["files importing"] += 1
            lines_per_file[min(len(lines), 3)] += 1
        wheres = defaultdict(set)
        for line, where, module in walk.imports:
            by_where[where] += 1
            wheres[where].add(line)
        # Calls: through the imports, and by the called name alone.
        for line, where, short, written, value, owner in walk.calls:
            qualified = qualify(walk, written)
            resolved = QUALIFIED.get(qualified) if qualified else None
            hit_imports = resolved is not None and names_target(value, resolved)
            hit_name = names_target(value, short)
            if True:
                call_rows.append((path.relative_to(root), line, where, written, qualified, value, hit_imports, hit_name))
            if hit_imports:
                wheres[where].add(line)
                if owner: walk.owned[owner].add(line)
        if not wheres:
            continue
        module_wide = bool(wheres.get("module"))
        helper_wide = bool(wheres.get("helper") or wheres.get("class"))
        tc_wide = bool(wheres.get("type_checking"))
        for name in tests:
            # The reach under each option, for this test.
            own = bool(walk.owned.get(name))
            if module_wide:
                tests_by_reach["module import"] += 1
            elif helper_wide:
                tests_by_reach["helper or class import only"] += 1
            elif tc_wide:
                tests_by_reach["TYPE_CHECKING import only"] += 1
            elif own:
                tests_by_reach["a test-body import only"] += 1
        if not module_wide and helper_wide:
            helper_only.append((path.relative_to(root), sorted(wheres["helper"] | wheres["class"]), len(tests)))
        if not module_wide and not helper_wide and tc_wide:
            tc_only.append((path.relative_to(root), sorted(wheres["type_checking"]), len(tests)))

    out = sys.stdout.write
    out(f"test files: {totals['files']}, top-level tests: {totals['tests']}\n")
    out(f"files with an import line of pytest or _pytest: {totals['files importing']}\n\n")
    out("Q3  import lines of the target per file\n")
    for n in (1, 2, 3):
        out(f"    {'3+' if n == 3 else n}: {lines_per_file[n]} files\n")
    out("\nimport lines by where they sit\n")
    for where in ("module", "type_checking", "test", "helper", "class"):
        out(f"    {where}: {by_where[where]}\n")
    out("\nQ4/Q7  top-level tests by what reaches them (first match wins, in this order)\n")
    for key in ("module import", "helper or class import only", "TYPE_CHECKING import only", "a test-body import only"):
        out(f"    {key}: {tests_by_reach[key]}\n")
    out("\nQ4  files whose only import of the target is in a helper or a class\n")
    for row in helper_only or [("none",)]:
        out(f"    {row}\n")
    out("\nQ7  files whose only import of the target is under TYPE_CHECKING\n")
    for row in tc_only or [("none",)]:
        out(f"    {row}\n")
    out("\nQ6  calls of a known importing function (resolved through the imports), or of a known short name\n")
    out("    path:line where | written | qualified | first argument | hit through imports | hit by name alone\n")
    hits_i = sum(r[6] for r in call_rows)
    hits_n = sum(r[7] for r in call_rows)
    for r in call_rows:
        if r[6] or r[7]:
            out(f"    {r[0]}:{r[1]} {r[2]} | {r[3]} | {r[4]} | {r[5]!r} | {r[6]} | {r[7]}\n")
    out(f"    known calls resolved through the imports: {sum(1 for r in call_rows if QUALIFIED.get(r[4]) )}\n")
    out(f"    hits through the imports: {hits_i}; hits by name alone: {hits_n}\n")
    out(f"    by name alone, not through the imports: {sum(1 for r in call_rows if r[7] and not r[6])}\n")
    looks = Counter(f"{r[3]} -> {r[4]}" for r in call_rows if not QUALIFIED.get(r[4]))
    out(f"    calls of a known short name that do not resolve to a known function: {sum(looks.values())}\n")
    for key, n in looks.most_common():
        out(f"        {n} x {key}\n")
    known = Counter(f"{r[4]}" for r in call_rows if QUALIFIED.get(r[4]))
    for key, n in known.most_common():
        out(f"        resolved: {n} x {key}\n")
    out(f"    through the imports, not by name alone: {sum(1 for r in call_rows if r[6] and not r[7])}\n")


if __name__ == "__main__":
    main(pathlib.Path(sys.argv[1]))
