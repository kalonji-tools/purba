#!/usr/bin/env bash
# What pytest and unittest pick as a test file and as a test, on one small tree.
# usage: bash run.sh <python with pytest> <empty dir>
set -u
PY=$1
D=$2
mkdir -p "$D/tests/a" "$D/tests/b"
cd "$D" || exit 1

cat >tests/test_shop.py <<'EOF'
import unittest
from test_stock import test_shared        # an imported test name


def test_lend():
    assert make() == 1


def testable():                           # "test" prefix, no underscore
    assert True


def make():                               # a public helper
    return 1


def _private():                           # a private helper
    return 2


async def test_async():
    assert True


class TestShop:                           # pytest: a class named Test*
    def test_method(self):
        assert True


class Shop:                               # not named Test*
    def test_hidden(self):
        assert True


class ShopCase(unittest.TestCase):        # unittest: a TestCase subclass
    def test_case(self):
        self.assertTrue(True)

    def helper(self):
        pass
EOF

cat >tests/test_stock.py <<'EOF'
def test_shared():
    assert True
EOF

cat >tests/stock_test.py <<'EOF'
def test_suffix():
    assert True
EOF

cat >tests/helpers.py <<'EOF'
def test_in_helper_module():              # a test name in a file with no marker
    assert True
EOF

cat >tests/testing_utils.py <<'EOF'
import unittest
print("testing_utils.py was imported")


class UtilCase(unittest.TestCase):
    def test_util(self):
        self.assertTrue(True)
EOF

# Two folders, each with a helper module of the same name, and no __init__.py.
for x in a b; do
  echo "WHO = '$x'" >tests/$x/helper.py
  cat >tests/$x/test_who_$x.py <<EOF
import helper


def test_helper_is_$x():
    assert helper.WHO == '$x'
EOF
done

# Two folders, each with a test file of the same name, and no __init__.py.
mkdir -p same/a same/b
for x in a b; do printf 'def test_one():\n    assert True\n' >same/$x/test_same.py; done

echo "== pytest --version"
"$PY" -I -m pytest --version
echo "== pytest --collect-only -q tests (pytest's defaults)"
"$PY" -I -m pytest -p no:cacheprovider --collect-only -q tests 2>&1 | tail -n +1
echo "== pytest -q tests/a tests/b (a helper.py in each folder)"
"$PY" -I -m pytest -p no:cacheprovider -q tests/a tests/b 2>&1 | tail -n 8
echo "== pytest -q same (two test files of one name)"
"$PY" -I -m pytest -p no:cacheprovider -q same 2>&1 | grep -E 'import file mismatch|error|passed' | head -5
echo "== python -m unittest discover -v (start at ., default pattern test*.py)"
"$PY" -I -m unittest discover -v 2>&1 | tail -n 5
echo "== python -m unittest discover -v -s tests -t tests"
"$PY" -I -m unittest discover -v -s tests -t tests 2>&1
