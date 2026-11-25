#!/usr/bin/env bash
VENV=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/96b6d852-95a0-47f0-872d-fb178e3ab17c/scratchpad/r1245/venv-a
# test-files under [tool.purba]: each value, and the path a tester names.
set -u
WORK=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/96b6d852-95a0-47f0-872d-fb178e3ab17c/scratchpad/r1245/work/$(basename "$(dirname "$(realpath "$0")")")
rm -rf "$WORK"
mkdir -p "$WORK"
export HOME=$WORK
unset PURBA_IMPLICIT_ROOT

# project <folder> <the line of test-files, or nothing>
project() {
  mkdir -p "$WORK/$1/tests"
  printf 'def test_add():\n    assert 1 + 1 == 2, "one and one make two"\n' >"$WORK/$1/tests/test_a.py"
  printf 'def test_sub():\n    assert 2 - 1 == 1, "two less one is one"\n' >"$WORK/$1/tests/b_test.py"
  printf '[project]\nname = "p"\n\n[tool.purba]\n%s\n' "$2" >"$WORK/$1/pyproject.toml"
  printf '\n# ~/%s/pyproject.toml\n' "$1"
  cat "$WORK/$1/pyproject.toml"
  printf '# ~/%s/tests holds test_a.py and b_test.py\n' "$1"
}

run() {
  local dir=$1
  shift
  printf '\n$ cd ~/%s && %s\n' "$dir" "${*//$VENV\/bin\//}"
  (cd "$WORK/$dir" && env "$@")
  printf '[exit %s]\n' "$?"
}

echo "purba $("$VENV/bin/python" -c 'import importlib.metadata as m; print(m.version("purba"))') from $VENV"

project no-key ''
run no-key "$VENV/bin/purba"

project suffix 'test-files = ["*_test.py"]'
run suffix "$VENV/bin/purba"
run suffix "$VENV/bin/purba" tests/test_a.py

project two 'test-files = ["test_*.py", "*_test.py"]'
run two "$VENV/bin/purba"

project glob 'test-files = ["test*.py"]'
run glob "$VENV/bin/purba"
run glob/tests "$VENV/bin/purba"

project string 'test-files = "*_test.py"'
run string "$VENV/bin/purba"

project number 'test-files = 5'
run number "$VENV/bin/purba"

project empty 'test-files = []'
run empty "$VENV/bin/purba"

project two-items 'test-files = ["tset_*.py", "test.py"]'
run two-items "$VENV/bin/purba"

project bad-items 'test-files = ["tset_*.py", "tests_*.py", 7]'
run bad-items "$VENV/bin/purba"
