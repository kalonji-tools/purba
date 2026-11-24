#!/usr/bin/env bash
# Which Ruff rules read the name marker "test", on a test in purba's form.
# usage: bash run.sh <ruff 0.17.6> <empty dir>
set -u
RUFF=$1
D=$2
cd "$D" || exit 1
cat >test_lend.py <<'EOF'
def use(factory):
    return factory


def shop():
    return 1


def test_lend(s: int = use(shop)) -> None:
    """Returns the book."""
    assert s == 1


def check_lend(s: int = use(shop)) -> None:
    """Returns the book."""
    assert s == 1
EOF
echo "== ruff --version"
"$RUFF" --version
echo "== ruff check --select PT028,D401 --no-cache --isolated test_lend.py"
"$RUFF" check --select PT028,D401 --no-cache --isolated --output-format concise test_lend.py 2>&1
