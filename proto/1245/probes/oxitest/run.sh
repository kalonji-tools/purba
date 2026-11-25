#!/usr/bin/env bash
VENV=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/96b6d852-95a0-47f0-872d-fb178e3ab17c/scratchpad/r1245/venv-a
# oxitest 4.0.0: what it prints about its root on the layouts of root-a, and
# what it does with python_files under [tool.oxitest].
set -u
WORK=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/96b6d852-95a0-47f0-872d-fb178e3ab17c/scratchpad/r1245/work/$(basename "$(dirname "$(realpath "$0")")")
rm -rf "$WORK"
mkdir -p "$WORK"
export HOME=$WORK

# project <folder> [pyproject.toml text]
project() {
  mkdir -p "$WORK/$1/tests"
  printf 'def test_add():\n    assert 1 + 1 == 2, "one and one make two"\n' >"$WORK/$1/tests/test_a.py"
  printf 'def test_sub():\n    assert 2 - 1 == 1, "two less one is one"\n' >"$WORK/$1/tests/b_test.py"
  if [ $# -gt 1 ]; then
    printf '%s\n' "$2" >"$WORK/$1/pyproject.toml"
    printf '\n# ~/%s/pyproject.toml\n' "$1"
    cat "$WORK/$1/pyproject.toml"
  fi
  printf '# ~/%s/tests holds test_a.py and b_test.py\n' "$1"
}

run() {
  local dir=$1
  shift
  printf '\n$ cd ~/%s && %s\n' "$dir" "${*//$VENV\/bin\//}"
  (
    cd "$WORK/$dir" && timeout 120 env "$@" 2>&1 | sed "s#$WORK#~#g"
    exit "${PIPESTATUS[0]}"
  )
  printf '[exit %s]\n' "$?"
}

echo "$("$VENV/bin/oxitest" --version) from $VENV"
printf '\n$ oxitest --help\n'
"$VENV/bin/oxitest" --help
printf '\n$ oxitest run --help | grep -iE "root|pattern|python_files|config|pyproject"\n'
"$VENV/bin/oxitest" run --help | grep -iE "root|pattern|python_files|config|pyproject"
printf '\n# Docs shipped in the venv: files under oxitest/ that are not .py, .pyi or .so\n'
(cd "$VENV/lib/python3.12/site-packages" && find oxitest oxitest-4.0.0.dist-info -type f ! -name '*.py' ! -name '*.pyi' ! -name '*.so' ! -name '*.pyc' | sort)
printf '\n# Config keys in the extension (strings near "python_files")\n'
strings "$VENV/lib/python3.12/site-packages/oxitest/_oxitest.cpython-312-x86_64-linux-gnu.so" |
  grep -oE 'testpathspython_files[a-z_]*|invalid glob pattern in python_files: ' | sort -u

# The root on each layout, from the folder and from a subfolder.
project with-project '[project]
name = "p"'
run with-project "$VENV/bin/oxitest" run -v --color never
run with-project/tests "$VENV/bin/oxitest" run -v --color never

project only-git
git -C "$WORK/only-git" init -q
run only-git "$VENV/bin/oxitest" run -v --color never
run only-git/tests "$VENV/bin/oxitest" run -v --color never

project neither
run neither "$VENV/bin/oxitest" run -v --color never
run neither/tests "$VENV/bin/oxitest" run -v --color never

# python_files under [tool.oxitest].
for case in 'suffix:python_files = ["*_test.py"]' \
  'glob:python_files = ["test*.py"]' \
  'string:python_files = "*_test.py"' \
  'empty:python_files = []' \
  'bad-glob:python_files = ["test_[.py"]' \
  'unknown-key:python_filez = ["*_test.py"]'; do
  name=${case%%:*}
  project "$name" "[project]
name = \"p\"

[tool.oxitest]
${case#*:}"
  run "$name" "$VENV/bin/oxitest" run -v --color never
done
