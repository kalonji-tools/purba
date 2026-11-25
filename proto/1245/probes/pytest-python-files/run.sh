#!/usr/bin/env bash
VENV=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/96b6d852-95a0-47f0-872d-fb178e3ab17c/scratchpad/r1245/venv-a
# pytest 9.2.0's reaction to python_files under [tool.pytest.ini_options],
# with and without --strict-config.
set -u
WORK=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/96b6d852-95a0-47f0-872d-fb178e3ab17c/scratchpad/r1245/work/$(basename "$(dirname "$(realpath "$0")")")
rm -rf "$WORK"
mkdir -p "$WORK"
export HOME=$WORK

# project <folder> <the line of python_files>
project() {
  mkdir -p "$WORK/$1/tests"
  printf 'def test_add():\n    assert 1 + 1 == 2, "one and one make two"\n' >"$WORK/$1/tests/test_a.py"
  printf 'def test_sub():\n    assert 2 - 1 == 1, "two less one is one"\n' >"$WORK/$1/tests/b_test.py"
  printf '[project]\nname = "p"\n\n[tool.pytest.ini_options]\n%s\n' "$2" >"$WORK/$1/pyproject.toml"
  printf '\n# ~/%s/pyproject.toml\n' "$1"
  cat "$WORK/$1/pyproject.toml"
  printf '# ~/%s/tests holds test_a.py and b_test.py\n' "$1"
}

run() {
  local dir=$1
  shift
  printf '\n$ cd ~/%s && %s\n' "$dir" "${*//$VENV\/bin\//}"
  (
    cd "$WORK/$dir" && env "$@" 2>&1 | sed "s#$WORK#~#g"
    exit "${PIPESTATUS[0]}"
  )
  printf '[exit %s]\n' "$?"
}

echo "pytest $("$VENV/bin/pytest" --version 2>&1) from $VENV"

for case in 'glob:python_files = ["test*.py"]' \
  'suffix:python_files = ["*_test.py"]' \
  'string:python_files = "*_test.py"' \
  'empty:python_files = []' \
  'number:python_files = 5' \
  'unknown-key:python_filez = ["*_test.py"]'; do
  name=${case%%:*}
  project "$name" "${case#*:}"
  run "$name" "$VENV/bin/pytest" -p no:cacheprovider -v
  run "$name" "$VENV/bin/pytest" -p no:cacheprovider -v --strict-config
done
