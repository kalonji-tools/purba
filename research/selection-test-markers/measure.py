"""Measure the markers of test files and test functions in the 9 pinned suites.

It reads each file with `ast` and imports nothing from the suites.

usage: python -I measure.py <suites dir> <out dir>

It writes files.tsv, functions.tsv, classes.tsv and bindings.tsv to <out dir>,
and prints every count that research/selection-test-markers.md quotes.
"""

import ast
import fnmatch
import os
import sys
from collections import Counter, defaultdict

SUITES = ["loguru", "attrs", "click", "requests", "dateutil", "pyflakes", "toolz", "six", "schedule"]

# The folder marker that the issue names: a folder named test, tests or testing.
TEST_DIRS = {"test", "tests", "testing"}

# pytest's default python_files, and the form's file marker.
PYTEST_FILES = ["test_*.py", "*_test.py"]
FORM_FILES = ["test_*.py"]
# unittest's default discover pattern.
UNITTEST_FILES = ["test*.py"]

# pytest's default norecursedirs.
NORECURSE = ["*.egg", ".*", "_darcs", "build", "CVS", "dist", "node_modules", "venv", "{arch}"]

# The command that each suite's own configuration names, read by hand.
# runner, roots (relative to the suite), file patterns, source of the row.
OWN = {
    "loguru": ("pytest", ["tests"], PYTEST_FILES, "pyproject.toml testpaths"),
    "attrs": ("pytest", ["tests"], PYTEST_FILES, "pyproject.toml testpaths"),
    "click": ("pytest", ["tests"], PYTEST_FILES, "pyproject.toml testpaths"),
    "requests": ("pytest", ["tests"], PYTEST_FILES, "pyproject.toml testpaths"),
    "dateutil": ("pytest", ["tests", "docs"], PYTEST_FILES + ["*_solution.py"], "tox.ini commands, pyproject.toml python_files"),
    "pyflakes": ("unittest", ["pyflakes"], UNITTEST_FILES, "tox.ini: unittest discover pyflakes"),
    "toolz": ("pytest", ["toolz"], PYTEST_FILES, "pyproject.toml testpaths"),
    "six": ("pytest", ["."], PYTEST_FILES, "tox.ini: python -m pytest, no testpaths"),
    "schedule": ("pytest", ["test_schedule.py", "schedule"], PYTEST_FILES, "tox.ini: py.test test_schedule.py schedule"),
}


def matches(name, patterns):
    return any(fnmatch.fnmatchcase(name, p) for p in patterns)


def walk(base):
    """Every .py file under base, as a path relative to base. Skips folders that start with '.'."""
    out = []
    for dirpath, dirnames, filenames in os.walk(base):
        dirnames[:] = sorted(d for d in dirnames if not d.startswith("."))
        for f in sorted(filenames):
            if f.endswith(".py"):
                out.append(os.path.relpath(os.path.join(dirpath, f), base))
    return out


def own_collects(base, rel):
    """Whether the suite's own command collects rel as a test file, read statically."""
    runner, roots, patterns, _ = OWN[os.path.basename(base)]
    name = os.path.basename(rel)
    parts = rel.split(os.sep)
    for root in roots:
        if root.endswith(".py"):
            if rel == root:
                return True
            continue
        if root != "." and not (rel == root or rel.startswith(root + os.sep)):
            continue
        inner = parts if root == "." else parts[len(root.split(os.sep)):]
        dirs = inner[:-1]
        if runner == "pytest":
            if any(matches(d, NORECURSE) for d in dirs):
                continue
        else:
            # unittest discover enters a folder only when it is a package.
            here = os.path.join(base, root)
            ok = True
            for d in dirs:
                here = os.path.join(here, d)
                if not os.path.exists(os.path.join(here, "__init__.py")):
                    ok = False
                    break
            if not ok:
                continue
        if matches(name, patterns):
            return True
    return False


def dotted(node):
    if isinstance(node, ast.Call):
        return dotted(node.func)
    if isinstance(node, ast.Attribute):
        return dotted(node.value) + "." + node.attr
    if isinstance(node, ast.Name):
        return node.id
    return type(node).__name__


def deco_kind(decorators):
    texts = [dotted(d) for d in decorators]
    if not texts:
        return "none", texts
    if any(t.split(".")[-1] in ("fixture", "yield_fixture") for t in texts):
        return "fixture", texts
    if all(".mark." in t or t.startswith("mark.") for t in texts):
        return "mark", texts
    return "other", texts


def required(args):
    pos = args.posonlyargs + args.args
    n = len(pos) - len(args.defaults)
    n += sum(1 for d in args.kw_defaults if d is None)
    return n


def name_class(name):
    if name.startswith("__") and name.endswith("__") and len(name) > 4:
        return "dunder"
    if name.startswith("test"):
        return "test"
    if name.startswith("_"):
        return "private"
    return "public"


def loads(tree):
    return Counter(n.id for n in ast.walk(tree) if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Load))


def imported_modules(tree):
    """Each module name that an import of the tree names, as dotted text; a relative one keeps its dots."""
    out = set()
    for n in ast.walk(tree):
        if isinstance(n, ast.Import):
            for a in n.names:
                out.add(a.name)
        elif isinstance(n, ast.ImportFrom):
            mod = "." * n.level + (n.module or "")
            out.add(mod)
            for a in n.names:
                out.add((mod + "." if not mod.endswith(".") else mod) + a.name)
    return out


def parse(path):
    with open(path, "rb") as fh:
        return ast.parse(fh.read(), path)


def main():
    root, outdir = sys.argv[1], sys.argv[2]
    files_rows, func_rows, class_rows, bind_rows = [], [], [], []
    for suite in SUITES:
        base = os.path.join(root, suite)
        rels = walk(base)
        tests = [r for r in rels if matches(os.path.basename(r), PYTEST_FILES)]
        test_dirs_of = defaultdict(int)
        for r in tests:
            test_dirs_of[os.path.dirname(r)] += 1
        trees = {}
        mods_imported = set()
        for r in tests:
            trees[r] = parse(os.path.join(base, r))
            mods_imported |= imported_modules(trees[r])
        last_parts = {m.split(".")[-1] for m in mods_imported if m.strip(".")}
        for r in rels:
            name = os.path.basename(r)
            d = os.path.dirname(r)
            dparts = d.split(os.sep) if d else []
            is_test = r in trees
            if is_test:
                kind = "test"
            elif name == "conftest.py":
                kind = "conftest"
            elif name == "__init__.py":
                kind = "init"
            else:
                kind = "other"
            stem = name[:-3]
            try:
                tree = trees.get(r) or parse(os.path.join(base, r))
                ok = True
            except SyntaxError:
                tree, ok = None, False
            top_public = top_test = 0
            if tree is not None:
                for n in tree.body:
                    if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)):
                        c = name_class(n.name)
                        top_public += c == "public"
                        top_test += c == "test"
            files_rows.append({
                "suite": suite,
                "path": r,
                "kind": kind,
                "form_name": int(matches(name, FORM_FILES)),
                "pytest_name": int(matches(name, PYTEST_FILES)),
                "unittest_name": int(matches(name, UNITTEST_FILES)),
                "under_test_dir": int(any(p in TEST_DIRS for p in dparts)),
                "under_test_prefixed_dir": int(any(p.startswith("test") for p in dparts)),
                "beside_test_file": int(not is_test and test_dirs_of.get(d, 0) > 0),
                "own_collects": int(own_collects(base, r)),
                "imported_by_a_test_file": int(not is_test and stem in last_parts),
                "parses": int(ok),
                "top_public_defs": top_public,
                "top_test_defs": top_test,
            })
        for r, tree in trees.items():
            used = loads(tree)
            for n in tree.body:
                if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    kind, texts = deco_kind(n.decorator_list)
                    func_rows.append({
                        "suite": suite, "path": r, "line": n.lineno, "name": n.name,
                        "class": name_class(n.name),
                        "test_not_test_": int(n.name.startswith("test") and not n.name.startswith("test_")),
                        "async": int(isinstance(n, ast.AsyncFunctionDef)),
                        "decorators": kind, "decorator_text": ",".join(texts),
                        "required_params": required(n.args),
                        "loaded_by_name": used.get(n.name, 0),
                    })
                elif isinstance(n, ast.ClassDef):
                    methods = [m for m in n.body if isinstance(m, (ast.FunctionDef, ast.AsyncFunctionDef))]
                    bases = ",".join(dotted(b) for b in n.bases)
                    class_rows.append({
                        "suite": suite, "path": r, "line": n.lineno, "name": n.name, "bases": bases,
                        "testcase_base": int(any(dotted(b).endswith("TestCase") for b in n.bases)),
                        "test_methods": sum(1 for m in methods if m.name.startswith("test")),
                        "other_methods": sum(1 for m in methods if not m.name.startswith("test")),
                    })
                # Each name that a statement at the module's level binds with no def.
                targets = []
                if isinstance(n, (ast.Assign, ast.AnnAssign, ast.AugAssign)):
                    ts = n.targets if isinstance(n, ast.Assign) else [n.target]
                    for t in ts:
                        for x in ast.walk(t):
                            if isinstance(x, ast.Name):
                                targets.append(("assign", x.id))
                elif isinstance(n, ast.Import):
                    for a in n.names:
                        targets.append(("import", (a.asname or a.name).split(".")[0]))
                elif isinstance(n, ast.ImportFrom):
                    for a in n.names:
                        if a.name != "*":
                            targets.append(("from-import", a.asname or a.name))
                for how, t in targets:
                    bind_rows.append({"suite": suite, "path": r, "line": n.lineno, "name": t, "how": how,
                                      "class": name_class(t)})
    write(os.path.join(outdir, "files.tsv"), files_rows)
    write(os.path.join(outdir, "functions.tsv"), func_rows)
    write(os.path.join(outdir, "classes.tsv"), class_rows)
    write(os.path.join(outdir, "bindings.tsv"), bind_rows)
    report(files_rows, func_rows, class_rows, bind_rows)


def write(path, rows):
    if not rows:
        open(path, "w").close()
        return
    keys = list(rows[0])
    with open(path, "w") as fh:
        fh.write("\t".join(keys) + "\n")
        for r in rows:
            fh.write("\t".join(str(r[k]) for k in keys) + "\n")


def table(title, header, rows):
    print(f"\n## {title}")
    print("\t".join(header))
    for r in rows:
        print("\t".join(str(x) for x in r))


def report(files, funcs, classes, binds):
    by = lambda rows, s: [r for r in rows if r["suite"] == s]  # noqa: E731
    total = lambda f: [f(s) for s in SUITES] + [f(None)]  # noqa: E731
    sel = lambda rows, s: rows if s is None else by(rows, s)  # noqa: E731
    file_of = {(r["suite"], r["path"]): r for r in files}

    def tf(s, pred=lambda r: True):
        return sum(1 for r in sel(files, s) if r["kind"] == "test" and pred(r))

    print("# Counts. Each count names its matcher.")
    print("# A 'test file' is a .py file whose name matches test_*.py or *_test.py (pytest's default python_files), anywhere in the suite.")
    print("# A 'test folder' is a folder named test, tests or testing, at any depth.")
    head = ["count"] + SUITES + ["total"]
    rows = [
        ["all .py files (folders that start with '.' skipped)"] + total(lambda s: len(sel(files, s))),
        ["all .py files under a test folder"] + total(lambda s: sum(1 for r in sel(files, s) if r["under_test_dir"])),
        ["test files (test_*.py or *_test.py)"] + total(lambda s: tf(s)),
        ["  of them named test_*.py (the form's marker)"] + total(lambda s: tf(s, lambda r: r["form_name"])),
        ["  of them under a test folder"] + total(lambda s: tf(s, lambda r: r["under_test_dir"])),
        ["  of them under no test folder"] + total(lambda s: tf(s, lambda r: not r["under_test_dir"])),
        ["  of them collected by the suite's own command"] + total(lambda s: tf(s, lambda r: r["own_collects"])),
        ["  under no test folder AND collected by own command"] + total(lambda s: tf(s, lambda r: not r["under_test_dir"] and r["own_collects"])),
        ["files the own command collects that are not test files"] + total(lambda s: sum(1 for r in sel(files, s) if r["kind"] != "test" and r["own_collects"])),
    ]
    table("1. Test files and test folders", head, rows)

    def other(s, pred):
        return sum(1 for r in sel(files, s) if r["kind"] != "test" and pred(r))

    rows = [
        ["other .py beside a test file (same folder)"] + total(lambda s: other(s, lambda r: r["beside_test_file"])),
        ["  __init__.py"] + total(lambda s: other(s, lambda r: r["beside_test_file"] and r["kind"] == "init")),
        ["  conftest.py"] + total(lambda s: other(s, lambda r: r["beside_test_file"] and r["kind"] == "conftest")),
        ["  other modules"] + total(lambda s: other(s, lambda r: r["beside_test_file"] and r["kind"] == "other")),
        ["    of them imported by a test file (by module name)"] + total(lambda s: other(s, lambda r: r["beside_test_file"] and r["kind"] == "other" and r["imported_by_a_test_file"])),
        ["    of them matching unittest's test*.py"] + total(lambda s: other(s, lambda r: r["beside_test_file"] and r["kind"] == "other" and r["unittest_name"])),
        ["other .py under a test folder, at any depth"] + total(lambda s: other(s, lambda r: r["under_test_dir"])),
        ["  not beside a test file (a deeper folder)"] + total(lambda s: other(s, lambda r: r["under_test_dir"] and not r["beside_test_file"])),
        ["  of all under a test folder: __init__ or conftest"] + total(lambda s: other(s, lambda r: r["under_test_dir"] and r["kind"] in ("init", "conftest"))),
        ["  public module-level defs in them"] + total(lambda s: sum(r["top_public_defs"] for r in sel(files, s) if r["kind"] != "test" and r["under_test_dir"])),
        ["  module-level test* defs in them"] + total(lambda s: sum(r["top_test_defs"] for r in sel(files, s) if r["kind"] != "test" and r["under_test_dir"])),
        ["  files of them that do not parse on 3.14"] + total(lambda s: other(s, lambda r: r["under_test_dir"] and not r["parses"])),
    ]
    table("2. Other .py files beside test files", head, rows)

    def fn(s, pred):
        return sum(1 for r in sel(funcs, s) if pred(r))

    def ff(s, pred):
        return len({r["path"] for r in sel(funcs, s) if pred(r)})

    pub = lambda r: r["class"] == "public"  # noqa: E731
    rows = [
        ["module-level defs in test files"] + total(lambda s: fn(s, lambda r: True)),
        ["  test* (pytest's default python_functions prefix)"] + total(lambda s: fn(s, lambda r: r["class"] == "test")),
        ["    of them async"] + total(lambda s: fn(s, lambda r: r["class"] == "test" and r["async"])),
        ["    of them with any decorator"] + total(lambda s: fn(s, lambda r: r["class"] == "test" and r["decorators"] != "none")),
        ["    of them with a fixture decorator"] + total(lambda s: fn(s, lambda r: r["class"] == "test" and r["decorators"] == "fixture")),
        ["    of them test* but not test_*"] + total(lambda s: fn(s, lambda r: r["test_not_test_"])),
        ["  public, not test*"] + total(lambda s: fn(s, pub)),
        ["    files holding one"] + total(lambda s: ff(s, pub)),
        ["    with a fixture decorator"] + total(lambda s: fn(s, lambda r: pub(r) and r["decorators"] == "fixture")),
        ["    with another decorator"] + total(lambda s: fn(s, lambda r: pub(r) and r["decorators"] in ("other", "mark"))),
        ["    with no decorator"] + total(lambda s: fn(s, lambda r: pub(r) and r["decorators"] == "none")),
        ["      of them with a required parameter"] + total(lambda s: fn(s, lambda r: pub(r) and r["decorators"] == "none" and r["required_params"] > 0)),
        ["      of them with no required parameter"] + total(lambda s: fn(s, lambda r: pub(r) and r["decorators"] == "none" and r["required_params"] == 0)),
        ["    async"] + total(lambda s: fn(s, lambda r: pub(r) and r["async"])),
        ["  private (_name, not dunder)"] + total(lambda s: fn(s, lambda r: r["class"] == "private")),
        ["    files holding one"] + total(lambda s: ff(s, lambda r: r["class"] == "private")),
        ["    with a fixture decorator"] + total(lambda s: fn(s, lambda r: r["class"] == "private" and r["decorators"] == "fixture")),
        ["  dunder"] + total(lambda s: fn(s, lambda r: r["class"] == "dunder")),
    ]
    table("3. Module-level functions in test files", head, rows)

    def cl(s, pred):
        return sum(1 for r in sel(classes, s) if pred(r))

    rows = [
        ["module-level classes in test files"] + total(lambda s: cl(s, lambda r: True)),
        ["  holding a test* method"] + total(lambda s: cl(s, lambda r: r["test_methods"] > 0)),
        ["    of them with a *TestCase base"] + total(lambda s: cl(s, lambda r: r["test_methods"] > 0 and r["testcase_base"])),
        ["  test* methods in them"] + total(lambda s: sum(r["test_methods"] for r in sel(classes, s))),
        ["  classes with no test* method"] + total(lambda s: cl(s, lambda r: r["test_methods"] == 0)),
    ]
    table("4. Classes in test files", head, rows)

    def bd(s, pred):
        return sum(1 for r in sel(binds, s) if pred(r))

    rows = [
        ["test* names bound with no def at module level"] + total(lambda s: bd(s, lambda r: r["class"] == "test")),
        ["  by an import or a from-import"] + total(lambda s: bd(s, lambda r: r["class"] == "test" and r["how"] != "assign")),
        ["  by an assignment"] + total(lambda s: bd(s, lambda r: r["class"] == "test" and r["how"] == "assign")),
        ["public names (not test*) bound by a from-import at module level"] + total(lambda s: bd(s, lambda r: r["class"] == "public" and r["how"] == "from-import")),
        ["  test files with one"] + total(lambda s: len({r["path"] for r in sel(binds, s) if r["class"] == "public" and r["how"] == "from-import"})),
        ["public names (not test*) bound by an assignment at module level"] + total(lambda s: bd(s, lambda r: r["class"] == "public" and r["how"] == "assign")),
    ]
    table("5. Names that a test file binds at module level with no def", head, rows)

    rows = [
        ["module-level test* defs in a test file under a test folder"] + total(lambda s: sum(1 for r in sel(funcs, s) if r["class"] == "test" and file_of[(r["suite"], r["path"])]["under_test_dir"])),
        ["  of them also under a folder whose name starts with test, below it"] + total(lambda s: sum(1 for r in sel(funcs, s) if r["class"] == "test" and sum(p.startswith("test") for p in os.path.dirname(r["path"]).split(os.sep)) >= 2)),
    ]
    table("6. Tests whose id carries the marker in the folder, the file and the function", head, rows)

    print("\n## 7. Lists")
    print("test files under no test folder:")
    for r in files:
        if r["kind"] == "test" and not r["under_test_dir"]:
            print(f"  {r['suite']}/{r['path']}\town_collects={r['own_collects']}")
    print("other modules beside a test file:")
    for r in files:
        if r["beside_test_file"] and r["kind"] == "other":
            print(f"  {r['suite']}/{r['path']}\timported={r['imported_by_a_test_file']}\tpublic_defs={r['top_public_defs']}")
    print("files collected by the own command that are not test files:")
    for r in files:
        if r["kind"] != "test" and r["own_collects"]:
            print(f"  {r['suite']}/{r['path']}")
    print("test* defs that are not test_*:")
    for r in funcs:
        if r["test_not_test_"]:
            print(f"  {r['suite']}/{r['path']}:{r['line']}\t{r['name']}")
    print("test* defs with a fixture decorator:")
    for r in funcs:
        if r["class"] == "test" and r["decorators"] == "fixture":
            print(f"  {r['suite']}/{r['path']}:{r['line']}\t{r['name']}")
    print("bindings of test* names with no def:")
    for r in binds:
        if r["class"] == "test":
            print(f"  {r['suite']}/{r['path']}:{r['line']}\t{r['name']}\t{r['how']}")


main()
