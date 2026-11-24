"""Measure, in the 9 pinned suites, what #1225's counts leave open for each strategy.

It reads each file with `ast`, `tomllib` and `configparser`, and imports nothing
from the suites.

usage: python -I measure.py <suites dir> <files.tsv of #1225> <out dir>

<files.tsv of #1225> is research/selection-test-markers/files.tsv at 93af24df.
It writes config_keys.tsv and hooks.tsv to <out dir>, and prints each count
that research/selection-strategies.md quotes.
"""

import ast
import configparser
import csv
import fnmatch
import os
import sys
import tomllib
from collections import Counter, defaultdict

SUITES = ["loguru", "attrs", "click", "requests", "dateutil", "pyflakes", "toolz", "six", "schedule"]

# pytest's default python_files: the file matcher of #1225.
PYTEST_FILES = ["test_*.py", "*_test.py"]

# The keys of pytest's configuration that change which files or which tests it collects.
PYTEST_KEYS = {"testpaths", "python_files", "python_classes", "python_functions", "norecursedirs", "addopts"}
# The flags inside addopts that change collection.
ADDOPTS_FLAGS = ("--ignore", "--ignore-glob", "--deselect", "--doctest-modules", "--doctest-glob", "-k", "-m", "--pyargs")
# The keys of coverage.py that name files.
COVERAGE_KEYS = {"source", "include", "omit"}
# The keys of Ruff that name files.
RUFF_KEYS = {"include", "exclude", "extend-exclude", "extend-include", "force-exclude"}

# The names that conftest.py or a test module defines to change collection.
CONFTEST_NAMES = {"collect_ignore", "collect_ignore_glob"}
HOOKS = {
    "pytest_ignore_collect", "pytest_collect_file", "pytest_collection_modifyitems",
    "pytest_pycollect_makeitem", "pytest_pycollect_makemodule", "pytest_generate_tests",
    "pytest_collect_directory",
}
COMPOUND = (ast.If, ast.For, ast.AsyncFor, ast.While, ast.With, ast.AsyncWith, ast.Try, ast.Match)
if hasattr(ast, "TryStar"):
    COMPOUND = COMPOUND + (ast.TryStar,)


def walk(base):
    out = []
    for dirpath, dirnames, filenames in os.walk(base):
        dirnames[:] = sorted(d for d in dirnames if not d.startswith("."))
        for f in sorted(filenames):
            if f.endswith(".py"):
                out.append(os.path.relpath(os.path.join(dirpath, f), base))
    return out


def parse(path):
    with open(path, "rb") as fh:
        return ast.parse(fh.read(), filename=path)


def config_rows(suite, base):
    """Each key of pytest, coverage and Ruff that names files or tests, in the suite's root files."""
    rows = []
    pp = os.path.join(base, "pyproject.toml")
    if os.path.exists(pp):
        with open(pp, "rb") as fh:
            data = tomllib.load(fh)
        tool = data.get("tool", {})
        for section in (("pytest", "ini_options"), ("pytest",)):
            node = tool
            for k in section:
                node = node.get(k, {}) if isinstance(node, dict) else {}
            for key, value in sorted(node.items()):
                if key in PYTEST_KEYS:
                    rows.append((suite, "pyproject.toml", "tool." + ".".join(section), key, value))
        cov = tool.get("coverage", {})
        for sub in ("run", "report"):
            for key, value in sorted(cov.get(sub, {}).items()):
                if key in COVERAGE_KEYS:
                    rows.append((suite, "pyproject.toml", "tool.coverage." + sub, key, value))
        ruff = tool.get("ruff", {})
        for key, value in sorted(ruff.items()):
            if key in RUFF_KEYS:
                rows.append((suite, "pyproject.toml", "tool.ruff", key, value))
        pfi = ruff.get("lint", {}).get("per-file-ignores", ruff.get("per-file-ignores", {}))
        for key in sorted(pfi):
            rows.append((suite, "pyproject.toml", "tool.ruff.lint.per-file-ignores", "glob", key))
    for fname, sections in (
        ("setup.cfg", {"tool:pytest": "pytest", "coverage:run": "coverage", "coverage:report": "coverage"}),
        ("tox.ini", {"pytest": "pytest", "coverage:run": "coverage", "coverage:report": "coverage"}),
        ("pytest.ini", {"pytest": "pytest"}),
        (".coveragerc", {"run": "coverage", "report": "coverage"}),
    ):
        path = os.path.join(base, fname)
        if not os.path.exists(path):
            continue
        cp = configparser.RawConfigParser()
        cp.read(path, encoding="utf-8")
        for section, kind in sections.items():
            if not cp.has_section(section):
                continue
            keys = PYTEST_KEYS if kind == "pytest" else COVERAGE_KEYS
            for key, value in cp.items(section):
                if key in keys:
                    rows.append((suite, fname, section, key, " ".join(value.split())))
    return rows


def addopts_flags(value):
    text = " ".join(value) if isinstance(value, list) else str(value)
    return [t for t in text.split() if t.split("=")[0] in ADDOPTS_FLAGS]


def hook_rows(suite, base, rel):
    """Each name that a conftest.py or a test module defines to change collection."""
    rows = []
    tree = parse(os.path.join(base, rel))
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name in HOOKS | {"load_tests"}:
            rows.append((suite, rel, "def", node.name, node.lineno))
        targets = []
        if isinstance(node, ast.Assign):
            targets = node.targets
        elif isinstance(node, (ast.AnnAssign, ast.AugAssign)):
            targets = [node.target]
        for t in targets:
            if isinstance(t, ast.Name) and t.id in CONFTEST_NAMES | {"__test__", "pytestmark"}:
                rows.append((suite, rel, "assign", t.id, node.lineno))
    # __test__ set on a function or a class, anywhere in the module.
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign):
            for t in node.targets:
                if isinstance(t, ast.Attribute) and t.attr == "__test__":
                    rows.append((suite, rel, "attr", "X.__test__", node.lineno))
        if isinstance(node, ast.ClassDef):
            for stmt in node.body:
                if isinstance(stmt, ast.Assign) and any(isinstance(t, ast.Name) and t.id == "__test__" for t in stmt.targets):
                    rows.append((suite, rel, "class-assign", node.name + ".__test__", stmt.lineno))
    return rows


def under_statement(tree):
    """Each `def test*` that sits under a compound statement of the module, and never in a def or a class."""
    found = []

    def visit(stmts, depth_under):
        for node in stmts:
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                if depth_under and node.name.startswith("test"):
                    found.append(node)
                continue
            if isinstance(node, ast.ClassDef):
                continue
            if isinstance(node, COMPOUND):
                for field in ("body", "orelse", "finalbody"):
                    visit(getattr(node, field, []) or [], True)
                for h in getattr(node, "handlers", []) or []:
                    visit(h.body, True)
                for c in getattr(node, "cases", []) or []:
                    visit(c.body, True)

    visit(tree.body, False)
    return found


def module_calls(tree, names):
    """Each module-level call to a name in names, outside each def and class."""
    hits = []

    def visit(node):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Lambda)):
            return
        if isinstance(node, ast.Call):
            f = node.func
            name = f.id if isinstance(f, ast.Name) else (f.attr if isinstance(f, ast.Attribute) else None)
            if name in names:
                hits.append((name, node.lineno))
        for child in ast.iter_child_nodes(node):
            visit(child)

    for stmt in tree.body:
        visit(stmt)
    return hits


def main():
    suites_dir, files_tsv, out = sys.argv[1], sys.argv[2], sys.argv[3]
    os.makedirs(out, exist_ok=True)

    # 1. Configuration keys that name test files or tests.
    rows = []
    for s in SUITES:
        rows += config_rows(s, os.path.join(suites_dir, s))
    with open(os.path.join(out, "config_keys.tsv"), "w", newline="") as fh:
        w = csv.writer(fh, delimiter="\t", lineterminator="\n")
        w.writerow(["suite", "file", "section", "key", "value"])
        for r in rows:
            w.writerow(r)
    print("== 1. configuration keys that name test files or tests (config_keys.tsv)")
    for kind, keys in (("pytest", PYTEST_KEYS), ("coverage", COVERAGE_KEYS), ("ruff", RUFF_KEYS | {"glob"})):
        per = defaultdict(list)
        for suite, _f, section, key, value in rows:
            if key in keys and (kind in section or (kind == "pytest" and section in ("tool:pytest", "pytest"))
                                or (kind == "coverage" and section in ("run", "report"))):
                per[suite].append(key)
        print(f"{kind}: suites that set a key = {len(per)}:", {s: sorted(Counter(v).items()) for s, v in per.items()})
    for suite, f, section, key, value in rows:
        if key == "addopts":
            print(f"addopts {suite}: collection flags = {addopts_flags(value)}")
    pyfiles = [(s, v) for s, _f, sec, k, v in rows if k == "python_files"]
    pyfuncs = [(s, v) for s, _f, sec, k, v in rows if k in ("python_functions", "python_classes")]
    print("python_files set by:", pyfiles)
    print("python_functions or python_classes set by:", pyfuncs)

    # 2. Hooks and names that change collection, in conftest.py and in test files.
    hooks = []
    test_files = defaultdict(list)
    for s in SUITES:
        base = os.path.join(suites_dir, s)
        for rel in walk(base):
            name = os.path.basename(rel)
            is_test = any(fnmatch.fnmatchcase(name, p) for p in PYTEST_FILES)
            if is_test:
                test_files[s].append(rel)
            if is_test or name == "conftest.py":
                hooks += hook_rows(s, base, rel)
    with open(os.path.join(out, "hooks.tsv"), "w", newline="") as fh:
        w = csv.writer(fh, delimiter="\t", lineterminator="\n")
        w.writerow(["suite", "path", "form", "name", "line"])
        for r in hooks:
            w.writerow(r)
    print("== 2. names that change collection, in conftest.py and test files (hooks.tsv)")
    print("rows:", len(hooks), "by name:", sorted(Counter(r[3] for r in hooks).items()))
    print("suites with a row:", sorted(Counter(r[0] for r in hooks).items()))

    # 3. Tests that a static reader of module-level registration or definition must find
    #    under a statement, and module-level loops and dynamic bindings.
    print("== 3. module-level shapes in the test files (matcher: test_*.py or *_test.py, anywhere)")
    tot = Counter()
    for s in SUITES:
        base = os.path.join(suites_dir, s)
        c = Counter()
        for rel in test_files[s]:
            tree = parse(os.path.join(base, rel))
            c["files"] += 1
            nested = under_statement(tree)
            c["def test* under a module-level statement"] += len(nested)
            c["module-level for or while"] += sum(isinstance(n, (ast.For, ast.AsyncFor, ast.While)) for n in tree.body)
            dyn = module_calls(tree, {"globals", "setattr", "exec", "locals", "vars"})
            c["module-level call to globals, locals, vars, setattr or exec"] += len(dyn)
            if nested:
                for n in nested:
                    print(f"  nested: {s}/{rel}:{n.lineno} {n.name}")
            for name, line in dyn:
                print(f"  dynamic: {s}/{rel}:{line} {name}()")
        print(s, dict(c))
        tot.update(c)
    print("total", dict(tot))

    # 4. Entries that a manifest needs, from #1225's files.tsv: one per collected file, or one per folder.
    print("== 4. manifest entries (matcher: own_collects = 1 in #1225's files.tsv)")
    folders = defaultdict(set)
    files = Counter()
    with open(files_tsv, newline="") as fh:
        for row in csv.DictReader(fh, delimiter="\t"):
            if row["own_collects"] == "1":
                files[row["suite"]] += 1
                folders[row["suite"]].add(os.path.dirname(row["path"]) or ".")
    for s in SUITES:
        print(s, "files", files[s], "folders", len(folders[s]), sorted(folders[s]))
    print("total files", sum(files.values()), "folders", sum(len(v) for v in folders.values()))


if __name__ == "__main__":
    main()
