"""Count test files, test shapes, imports and roots in the pinned repositories.

This is a matcher, not purba. It applies the patterns of purba's records, and
of unittest, to the paths and trees of each repository.

usage: python -I measure.py <repos dir> <repos.tsv> <out dir>
"""

import ast
import csv
import fnmatch
import gzip
import os
import re
import subprocess
import sys
import tomllib
from collections import Counter

REPOS, LIST, OUT = sys.argv[1:4]
CONFIGS = ("pyproject.toml", "setup.cfg", "tox.ini", "pytest.ini")


def purba_name(name):
    """purba's test file: `test_*.py` or `*_test.py`."""
    return fnmatch.fnmatchcase(name, "test_*.py") or fnmatch.fnmatchcase(name, "*_test.py")


def unittest_name(name):
    """unittest's default pattern, as `discover` reads it: `test*.py`."""
    return fnmatch.fnmatchcase(name, "test*.py")


def edits(a, b):
    """Edits from a to b: change, add or drop one letter, or swap two neighbours."""
    d = [[i + j if i * j == 0 else 0 for j in range(len(b) + 1)] for i in range(len(a) + 1)]
    for i in range(1, len(a) + 1):
        for j in range(1, len(b) + 1):
            cost = a[i - 1] != b[j - 1]
            d[i][j] = min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + cost)
            if i > 1 and j > 1 and a[i - 1] == b[j - 2] and a[i - 2] == b[j - 1]:
                d[i][j] = min(d[i][j], d[i - 2][j - 2] + 1)
    return d[len(a)][len(b)]


def near_name(name):
    """A file name that purba's patterns miss, and whose first word (up to the
    first `_`) or last word (after the last `_`) is at most one edit from
    `test`, case ignored. The rule of a name near `test` in purba's record,
    moved from a function's name to a file's name: a matcher, not purba."""
    if purba_name(name) or not name.endswith(".py"):
        return False
    stem = name[:-3].lower()
    words = stem.split("_")
    return edits(words[0], "test") <= 1 or (len(words) > 1 and edits(words[-1], "test") <= 1)


def name_class(name):
    """The kind of a name that `test*.py` matches and purba's patterns do not."""
    if name == "tests.py":
        return "tests.py"
    if name == "test.py":
        return "test.py"
    if name.startswith("tests_"):
        return "tests_*.py"
    return "test<other>*.py"


def tracked(repo):
    out = subprocess.run(["git", "-C", repo, "ls-files", "-z"], capture_output=True, check=True).stdout
    paths = [p for p in out.decode().split("\0") if p]
    # purba's stage one skips each hidden folder; git ls-files already leaves out ignored files.
    return [p for p in paths if not any(part.startswith(".") for part in p.split("/")[:-1])]


def base_name(node):
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        return node.attr
    return ""


def shape(path):
    """Count the tests of one file, by where they sit."""
    try:
        with open(path, "rb") as f:
            tree = ast.parse(f.read())
    except (SyntaxError, ValueError):
        return None
    top = 0
    unittest_methods = 0
    plain_methods = 0
    other_methods = 0
    imports = set()
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name.startswith("test"):
            top += 1
        elif isinstance(node, ast.ClassDef):
            methods = sum(
                1
                for n in node.body
                if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name.startswith("test")
            )
            if any(base_name(b).endswith("TestCase") for b in node.bases):
                unittest_methods += methods
            elif node.name.startswith("Test"):
                plain_methods += methods
            else:
                other_methods += methods
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            imports.update(a.name.split(".")[0] for a in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module and node.level == 0:
            imports.add(node.module.split(".")[0])
    return top, unittest_methods, plain_methods, other_methods, imports


def kind(top, methods):
    """A file's tests: top-level `test*` functions, `test*` methods of a
    module-level class, both, or neither."""
    if top and methods:
        return "both"
    if top:
        return "functions only"
    if methods:
        return "methods only"
    return "no test"


def has_table(pyproject, *keys):
    try:
        with open(pyproject, "rb") as f:
            data = tomllib.load(f)
    except (OSError, tomllib.TOMLDecodeError):
        return False
    for k in keys:
        if not isinstance(data, dict) or k not in data:
            return False
        data = data[k]
    return True


def ladder(repo, cwd):
    """The root ladder of the brief: (1) [tool.purba], then, with
    PURBA_IMPLICIT_ROOT set, (2) [project], (3) the folder of .git,
    (4) the working folder. The walk goes up from cwd and stops at .git."""
    walk = []
    d = cwd
    while True:
        walk.append(d)
        if d == "" or os.path.exists(os.path.join(repo, d, ".git")):
            break
        d = os.path.dirname(d)
    for d in walk:
        if has_table(os.path.join(repo, d, "pyproject.toml"), "tool", "purba"):
            return "1", d
    for d in walk:
        if has_table(os.path.join(repo, d, "pyproject.toml"), "project"):
            return "2", d
    return "3", walk[-1]


def doctest_lines(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as f:
            return any(re.match(r"\s*>>> ", line) for line in f)
    except OSError:
        return False


rows = []
near_lists = {}
counts = []
with open(LIST) as f:
    repos = list(csv.DictReader(f, delimiter="\t"))

for r in repos:
    name = r["name"]
    repo = os.path.join(REPOS, name)
    paths = tracked(repo)
    py = [p for p in paths if p.endswith(".py")]
    c = Counter()
    c["py files"] = len(py)
    unseen = []
    nears = []
    for p in py:
        b = os.path.basename(p)
        pm, um = purba_name(b), unittest_name(b)
        folders = p.split("/")[:-1]
        pkg = all(os.path.exists(os.path.join(repo, *folders[: i + 1], "__init__.py")) for i in range(len(folders)))
        s = shape(os.path.join(repo, p))
        if s is None:
            top, ut, plain, other, imports = 0, 0, 0, 0, set()
            k = "parse error"
        else:
            top, ut, plain, other, imports = s
            k = kind(top, ut + plain + other)
        if pm:
            c["P"] += 1
            c["P: " + k] += 1
        if um:
            c["U"] += 1
            c["U: " + k] += 1
            if pkg:
                c["U and a package path"] += 1
        if um and not pm:
            c["U not P"] += 1
            c["U not P: " + name_class(b)] += 1
            c["U not P: " + k] += 1
            c["U not P: " + name_class(b) + ": " + k] += 1
        if pm and not um:
            c["P not U"] += 1
            c["P not U: " + k] += 1
        if (pm or um) and k == "no test":
            c["P or U, no test"] += 1
        if near_name(b):
            c["near"] += 1
            if k in ("functions only", "methods only", "both"):
                c["near, with a test"] += 1
            else:
                nears.append(p)
        if b == "__init__.py" and k in ("functions only", "methods only", "both"):
            c["__init__.py with a test"] += 1
        if not (pm or um) and (top or ut) and b not in ("conftest.py", "__init__.py"):
            unseen.append(p)
        if pm or um:
            if "django" in imports:
                c["P or U: imports django"] += 1
            if imports & {"pytest", "_pytest"}:
                c["P or U: imports pytest"] += 1
            c["P or U: test functions"] += top
            c["P or U: TestCase methods"] += ut
            c["P or U: Test-class methods"] += plain
            c["P or U: other-class methods"] += other
            step, root = ladder(repo, os.path.dirname(p))
            rows.append([name, p, int(pm), int(um), int(pkg), k, top, ut, plain, other, int("django" in imports), int(bool(imports & {"pytest", "_pytest"})), step, root or "."])
        if doctest_lines(os.path.join(repo, p)):
            c["py files with a >>> line"] += 1
    c["files outside P and U that hold a test function or a TestCase method"] = len(unseen)
    for p in paths:
        if p.endswith((".txt", ".rst", ".md")) and doctest_lines(os.path.join(repo, p)):
            c["text files with a >>> line"] += 1
    for cfg in CONFIGS:
        fp = os.path.join(repo, cfg)
        if os.path.exists(fp):
            with open(fp, encoding="utf-8") as f:
                text = f.read()
            if "--doctest-modules" in text or "doctest_" in text or "--doctest-glob" in text:
                c["root config names a doctest option"] += 1
    pyprojects = [p for p in paths if os.path.basename(p) == "pyproject.toml"]
    c["pyproject.toml files"] = len(pyprojects)
    c["conftest.py files"] = sum(os.path.basename(p) == "conftest.py" for p in py)
    c["pyproject.toml with [project]"] = sum(has_table(os.path.join(repo, p), "project") for p in pyprojects)
    c["pyproject.toml with [tool.purba]"] = sum(has_table(os.path.join(repo, p), "tool", "purba") for p in pyprojects)
    step, root = ladder(repo, "")
    c["root step from the repository's root: " + step] = 1
    roots = Counter(row[-1] for row in rows if row[0] == name)
    c["distinct roots from the folders of P or U files"] = len(roots)
    counts.append((name, r["group"], c, unseen))
    near_lists[name] = nears

SUMMARY = [
    ("P", "P"),
    ("U", "U"),
    ("U_not_P", "U not P"),
    ("U_not_P_tests.py", "U not P: tests.py"),
    ("U_not_P_tests_x.py", "U not P: tests_*.py"),
    ("U_not_P_test.py", "U not P: test.py"),
    ("U_not_P_other", "U not P: test<other>*.py"),
    ("U_not_P_with_a_test", None),
    ("P_not_U", "P not U"),
    ("init_with_a_test", "__init__.py with a test"),
    ("near", "near"),
    ("near_with_a_test", "near, with a test"),
    ("P_functions_only", "P: functions only"),
    ("P_both", "P: both"),
    ("P_methods_only", "P: methods only"),
    ("P_no_test", "P: no test"),
    ("P_or_U_test_functions", "P or U: test functions"),
    ("P_or_U_test_methods", None),
    ("P_or_U_imports_django", "P or U: imports django"),
    ("P_or_U_imports_pytest", "P or U: imports pytest"),
    ("conftest_files", "conftest.py files"),
    ("pyproject_files", "pyproject.toml files"),
    ("root_step_from_repo_root", None),
    ("roots_from_test_folders", "distinct roots from the folders of P or U files"),
]

with open(os.path.join(OUT, "summary.tsv"), "w") as f:
    print("repo\tgroup\t" + "\t".join(k for k, _ in SUMMARY), file=f)
    for name, group, c, _ in counts:
        vals = []
        for k, key in SUMMARY:
            if k == "U_not_P_with_a_test":
                v = c["U not P: methods only"] + c["U not P: functions only"] + c["U not P: both"]
            elif k == "P_or_U_test_methods":
                v = c["P or U: TestCase methods"] + c["P or U: Test-class methods"] + c["P or U: other-class methods"]
            elif k == "root_step_from_repo_root":
                v = next(s[-1] for s in c if s.startswith("root step from the repository's root: "))
            else:
                v = c[key]
            vals.append(str(v))
        print(f"{name}\t{group}\t" + "\t".join(vals), file=f)

with open(os.path.join(OUT, "files.tsv.gz"), "wb") as raw, gzip.GzipFile(fileobj=raw, mode="wb", mtime=0, filename="") as gz:
    import io

    f = io.TextIOWrapper(gz, newline="")
    w = csv.writer(f, delimiter="\t", lineterminator="\n")
    w.writerow(["repo", "path", "purba_name", "unittest_name", "package_path", "shape", "test_functions", "testcase_methods", "test_class_methods", "other_class_methods", "imports_django", "imports_pytest", "root_step", "root"])
    w.writerows(rows)
    f.flush()
    f.detach()

with open(os.path.join(OUT, "counts.txt"), "w") as f:
    for name, group, c, unseen in counts:
        print(f"== {name} ({group})", file=f)
        for k in sorted(c):
            print(f"{k}\t{c[k]}", file=f)
        for p in sorted(near_lists[name]):
            print(f"  near, no test: {p}", file=f)
        for p in sorted(unseen)[:5]:
            print(f"  outside P and U: {p}", file=f)
print("python", sys.version.split()[0])
