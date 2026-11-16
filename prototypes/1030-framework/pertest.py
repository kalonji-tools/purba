"""Per test function in the test files under argv[1] that import pytest: does the test touch pytest?

Matcher: a module-level `def test*` (not in a class). It "touches pytest" when its
decorators or body hold a Name bound by `import pytest` / `from pytest import ...`,
or it has a parameter with no default (a fixture). Otherwise it is "clean".
"""
import ast
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
files = sorted(
    p for p in root.rglob("*.py") if p.name.startswith("test_") or p.name.endswith("_test.py")
)
tot = {"files": 0, "tests": 0, "touch": 0, "fixture_only": 0, "clean": 0, "files_with_clean": 0}
for f in files:
    try:
        tree = ast.parse(f.read_text())
    except (SyntaxError, UnicodeDecodeError):
        continue
    names = set()
    for node in tree.body:
        if isinstance(node, ast.Import):
            names |= {a.asname or a.name for a in node.names if a.name.split(".")[0] == "pytest"}
        elif isinstance(node, ast.ImportFrom) and (node.module or "").split(".")[0] == "pytest":
            names |= {a.asname or a.name for a in node.names}
    if not names:
        continue
    tot["files"] += 1
    clean_here = 0
    for node in tree.body:
        is_def = isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        if not is_def or not node.name.startswith("test"):
            continue
        tot["tests"] += 1
        used = any(isinstance(n, ast.Name) and n.id in names for n in ast.walk(node))
        a = node.args
        nodefault = len(a.args) - len(a.defaults) > 0
        if used:
            tot["touch"] += 1
        elif nodefault:
            tot["fixture_only"] += 1
        else:
            tot["clean"] += 1
            clean_here += 1
    tot["files_with_clean"] += clean_here > 0
for k, v in tot.items():
    print(f"{k:17} {v}")
