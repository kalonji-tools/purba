#!/usr/bin/env bash
# pytest's rootdir: which marker sets it, what it is with no marker, and
# whether the id of one test changes with the folder that pytest runs from.
# usage: bash run.sh <pytest 9.2.0> <empty dir>
set -u
PYTEST=$1
D=$2
export HOME=$D/home XDG_CONFIG_HOME=$D/home/.config
unset PYTEST_ADDOPTS
mkdir -p "$HOME" "$D/p/a" "$D/p/b" "$D/elsewhere"
printf 'def test_a():\n    pass\n' >"$D/p/a/test_a.py"
printf 'def test_b():\n    pass\n' >"$D/p/b/test_b.py"
echo "== pytest --version"
"$PYTEST" --version 2>&1

# show <cwd> <args...>: the rootdir and configfile lines, then each id
show() {
  local cwd=$1
  shift
  echo "-- cwd ${cwd#"$D"/}: pytest $*"
  (cd "$cwd" && "$PYTEST" --collect-only -p no:cacheprovider "$@" 2>&1 |
    grep -E '^(rootdir|configfile):')
  (cd "$cwd" && "$PYTEST" --collect-only -q -p no:cacheprovider "$@" 2>&1 |
    grep '::')
}

echo "== 1. no marker anywhere"
show "$D/p"
show "$D/p/a"
show "$D/p/a" . ../b
show "$D/elsewhere" ../p/a ../p/b

echo "== 2. p/.git only: git does not mark the rootdir"
mkdir -p "$D/p/.git"
show "$D/p/a"

echo "== 3. p/setup.py"
printf 'from setuptools import setup\n' >"$D/p/setup.py"
show "$D/p/a"
rm "$D/p/setup.py"

echo "== 4. p/pyproject.toml with no [tool.pytest] table"
printf '[project]\nname = "p"\n' >"$D/p/pyproject.toml"
show "$D/p/a"

echo "== 5. p/pyproject.toml with [tool.pytest]"
printf '[tool.pytest]\n' >"$D/p/pyproject.toml"
show "$D/p"
show "$D/p/a"

echo "== 6. --rootdir=b, run from p"
show "$D/p" --rootdir=b

echo "== 7. -c /dev/null, run from p/a: no configuration, and no rootdir from the file"
show "$D/p/a" -c /dev/null
