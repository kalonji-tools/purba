#!/usr/bin/env bash
# Prototype harness for #1229. This branch is never merged.
#
#   proto/1229/run.sh <purba> <out-dir> <work-dir> <label> [main]
#
# Writes the probe project shop/ under <work-dir>, with one case of each
# scenario of the issue, then runs the installed <purba> in it: once on the
# folder `.` and once on `tests/test_lend.py`, for each strategy. In `main`
# mode it runs each command once, with no switch. Each run lands in
# <out-dir>/<strategy>.<target>.txt, and the tree in <out-dir>/tree.txt.
set -euo pipefail

purba=$1
out=$2
work=$3
label=$4
mode=${5:-proto}
mkdir -p "$out" "$work"
shop=$work/shop
rm -rf "$shop"
mkdir -p "$shop/tests/samples" "$shop/extra"

cat >"$shop/pyproject.toml" <<'EOF'
[project]
name = "shop"

[tool.purba]
tests = ["tests"]
EOF

# A test file at the root, outside any test folder.
cat >"$shop/test_root.py" <<'EOF'
def test_root_sum():
    assert 1 + 1 == 2
EOF

# A test file: a public helper, a misspelled test, a test named tests_…, a
# helper that asserts, and a name that an import binds.
cat >"$shop/tests/test_lend.py" <<'EOF'
from json import dumps


def make_book():
    return {"title": "Dune", "lent": False}


def test_lend():
    book = make_book()
    assert book["title"] == "Dune"


def tset_checkout():
    assert 2 + 2 == 4


def tests_return():
    assert 3 * 1 == 3


def assert_titled(book):
    assert book["title"] == "Dune"


def test_dumps():
    assert dumps(1) == "1"
EOF

# A helper module beside the test file.
cat >"$shop/tests/helpers.py" <<'EOF'
def shelf():
    return ["Dune"]
EOF

# A file of pytest beside the tests.
cat >"$shop/tests/conftest.py" <<'EOF'
import pytest


@pytest.fixture
def book():
    return {"title": "Dune"}
EOF

# A sample script in a folder under tests/.
cat >"$shop/tests/samples/script.py" <<'EOF'
def test_sample():
    print("a sample script, run by hand")
EOF

# A test file with a misspelled name, beside the tests.
cat >"$shop/tests/tset_typo.py" <<'EOF'
def test_typo():
    assert 1 == 1
EOF

# A test file that no entry of the list names.
cat >"$shop/extra/test_extra.py" <<'EOF'
def test_extra():
    assert 2 * 3 == 6
EOF

{
  echo "probe project <tmp>/shop, written by proto/1229/run.sh"
  (cd "$shop" && find . -type f | LC_ALL=C sort | while read -r file; do
    echo "--- $file"
    cat "$file"
  done)
} >"$out/tree.txt"

# go <strategy or -> <target>
go() {
  local strategy=$1 target=$2
  local name=${target//\//_}
  [[ $target == . ]] && name=dot
  local file="$out/$strategy.$name.txt"
  [[ $strategy == - ]] && file="$out/main.$name.txt"
  local -a vars=("PATH=/run/current-system/sw/bin:/usr/bin:/bin" "HOME=$work/home")
  [[ $strategy != - ]] && vars+=("PURBA_PROTO_MARKERS=$strategy")
  local start end status
  start=$(date +%s.%N)
  set +e
  (cd "$shop" && env -i "${vars[@]}" timeout 60 "$purba" "$target" \
    >"$work/stdout" 2>"$work/stderr")
  status=$?
  set -e
  end=$(date +%s.%N)
  {
    echo "purba: $label"
    echo "env: ${vars[*]:2}"
    echo "command: cd <tmp>/shop && purba $target"
    echo "--- stdout"
    cat "$work/stdout"
    echo "--- stderr"
    cat "$work/stderr"
    echo "--- exit $status"
    awk -v a="$start" -v b="$end" 'BEGIN { printf "--- seconds %.1f\n", b - a }'
  } | sed "s#$work#<tmp>#g" >"$file"
}

mkdir -p "$work/home"
if [[ $mode == main ]]; then
  go - .
  go - tests/test_lend.py
else
  for strategy in baseline name name-and-check visibility folder list-and-check; do
    go "$strategy" .
    go "$strategy" tests/test_lend.py
  done
fi
echo "runs in $out"
