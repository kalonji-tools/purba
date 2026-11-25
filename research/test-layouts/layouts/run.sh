#!/usr/bin/env bash
# Which files and tests pytest and unittest find in a tree that holds one
# tests.py, one test.py, a tests/ folder, a doctest in a module and a doctest
# in a text file.
# usage: bash run.sh <python with pytest 9.2.0> <empty dir>
set -u
PY=$1
D=$2
cd "$D" || exit 1
echo "== versions"
"$PY" -I -c 'import sys, pytest; print("python", sys.version.split()[0], "pytest", pytest.__version__)'

mkdir -p pkg tests docs
touch pkg/__init__.py
cat >tests.py <<'EOF'
import unittest
def test_in_tests_py():
    pass
class InTestsPy(unittest.TestCase):
    def test_method(self):
        pass
EOF
printf 'def test_in_test_py():\n    pass\n' >test.py
printf 'def test_lend():\n    pass\n' >tests/test_lend.py
printf 'def test_suffix():\n    pass\n' >tests/lend_test.py
printf 'def helper():\n    return 1\n' >tests/testing_helpers.py
cat >pkg/calc.py <<'EOF'
def add(a, b):
    """
    >>> add(1, 2)
    3
    """
    return a + b
EOF
printf 'The usage.\n\n>>> 1 + 1\n2\n' >docs/usage.txt
find . -type f | sort

# -I leaves the working folder off sys.path; pytest's rootdir import mode puts it back.
echo "== 1. pytest --collect-only, with no option"
"$PY" -I -m pytest --collect-only -q -p no:cacheprovider 2>&1 | grep -E '::|test|collected'
echo "== 2. pytest --collect-only --doctest-modules"
"$PY" -I -m pytest --collect-only -q -p no:cacheprovider --doctest-modules 2>&1 | grep -E '::|collected'
echo "== 3. pytest --collect-only --doctest-glob='*.txt'"
"$PY" -I -m pytest --collect-only -q -p no:cacheprovider --doctest-glob='*.txt' 2>&1 | grep -E '::|txt|collected'
echo "== 4. unittest discover, from the root: tests/ has no __init__.py"
"$PY" -I -m unittest discover -v 2>&1 | grep -E '\.\.\.|^Ran'
echo "== 5. unittest discover, from the root, after touch tests/__init__.py: tests.py and tests/ share a name"
touch tests/__init__.py
"$PY" -I -m unittest discover -v 2>&1 | grep -E '\.\.\.|^Ran|^ImportError'
rm tests/__init__.py
echo "== 6. python -m doctest -v pkg/calc.py docs/usage.txt"
"$PY" -I -m doctest -v pkg/calc.py docs/usage.txt 2>&1 | grep -E 'passed|failed'
echo "== 7. the names that test_*.py or *_test.py match, by fnmatch: a matcher, not purba"
"$PY" -I -c "
import fnmatch, os
for root, dirs, files in sorted(os.walk('.')):
    for f in sorted(files):
        if fnmatch.fnmatchcase(f, 'test_*.py') or fnmatch.fnmatchcase(f, '*_test.py'):
            print(os.path.join(root, f))
"
