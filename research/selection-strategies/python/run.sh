#!/usr/bin/env bash
# What Python's own discovery does with a missing marker and with a helper that
# carries one: `from m import *` with and without `__all__`, entry points,
# unittest's load_tests, and pytest's `__test__` and `python_functions`.
# usage: bash run.sh <python 3.13 with pytest 9.2.0> <empty dir>
set -u
PY=$1
D=$2
cd "$D" || exit 1
echo "== python and pytest"
"$PY" -I -c 'import sys; print(sys.version.split()[0])'
"$PY" -I -m pytest --version

echo "== 1. from m import *: with no __all__, each public name; a _name stays out"
mkdir star && cd star || exit 1
printf 'def lend(): pass\ndef _helper(): pass\nimport os\n' >noall.py
printf '__all__ = ["lend"]\ndef lend(): pass\ndef helper(): pass\n' >withall.py
printf '__all__ = ["lend", "gone"]\ndef lend(): pass\n' >badall.py
"$PY" -I -c 'import sys; sys.path.insert(0, "."); ns = {}; exec("from noall import *", ns); print(sorted(k for k in ns if k != "__builtins__"))'
echo "== 2. with __all__, only the listed names; a public helper stays out"
"$PY" -I -c 'import sys; sys.path.insert(0, "."); ns = {}; exec("from withall import *", ns); print(sorted(k for k in ns if k != "__builtins__"))'
echo "== 3. __all__ that lists a name the module does not bind"
"$PY" -I -c 'import sys; sys.path.insert(0, "."); exec("from badall import *", {})' 2>&1 | tail -1
cd .. || exit 1

echo "== 4. entry points: a group lists each unit; the loader finds only what is listed"
mkdir -p ep/shop ep/shop-1.0.dist-info && cd ep || exit 1
printf 'def lend(): return "lend"\ndef unlisted(): return "unlisted"\n' >shop/plugins.py
touch shop/__init__.py
printf 'Metadata-Version: 2.1\nName: shop\nVersion: 1.0\n' >shop-1.0.dist-info/METADATA
printf '[shop.actions]\nlend = shop.plugins:lend\ngone = shop.plugins:gone\n' >shop-1.0.dist-info/entry_points.txt
"$PY" -I - <<'EOF'
import sys
sys.path.insert(0, ".")
from importlib.metadata import entry_points
eps = entry_points(group="shop.actions")
print("listed:", sorted(e.name for e in eps))
for e in sorted(eps, key=lambda e: e.name):
    try:
        print(e.name, "->", e.load()())
    except Exception as exc:
        print(e.name, "-> load fails:", type(exc).__name__, exc)
EOF
cd .. || exit 1

echo "== 5. unittest: load_tests replaces discovery's own choice for its module"
mkdir lt && cd lt || exit 1
cat >test_shop.py <<'EOF'
import unittest

class Lend(unittest.TestCase):
    def test_lend(self): pass
    def test_also(self): pass

def load_tests(loader, tests, pattern):
    return unittest.TestSuite([Lend("test_lend")])
EOF
"$PY" -I -m unittest discover -v 2>&1 | grep -E '^test|^Ran'
cd .. || exit 1

echo "== 6. pytest: __test__ = False hides a test; __test__ = True makes any function one"
mkdir pt && cd pt || exit 1
cat >test_shop.py <<'EOF'
def test_hidden(): pass
test_hidden.__test__ = False

def lend_works(): pass
lend_works.__test__ = True

class TestCaseLike:
    __test__ = False
    def test_in_class(self): pass

def test_plain(): pass
EOF
"$PY" -I -m pytest -q --collect-only -p no:cacheprovider 2>&1 | grep -E '::|collected|warning'

echo "== 7. pytest: python_functions = check_* replaces the default test prefix"
printf '[pytest]\npython_functions = check_*\n' >pytest.ini
printf 'def check_lend(): pass\ndef test_lend(): pass\n' >test_alt.py
"$PY" -I -m pytest -q --collect-only -p no:cacheprovider test_alt.py 2>&1 | grep -E '::|collected'
cd .. || exit 1

echo "== 8. doctest: a module's __test__ dict adds units that discovery would not find"
mkdir dt && cd dt || exit 1
cat >shop.py <<'EOF'
def lend():
    """
    >>> 1 + 1
    2
    """

__test__ = {"extra": """
>>> 2 + 2
4
"""}
EOF
"$PY" -I -c 'import sys; sys.path.insert(0, "."); import doctest, shop; print(sorted(t.name for t in doctest.DocTestFinder().find(shop) if t.examples))'
