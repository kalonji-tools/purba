#!/usr/bin/env bash
VENV=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/96b6d852-95a0-47f0-872d-fb178e3ab17c/scratchpad/r1245/venv-a
# pytest 9.2.0's header (rootdir, configfile) on the layouts of root-a, from
# the folder and from a subfolder.
set -u
WORK=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/96b6d852-95a0-47f0-872d-fb178e3ab17c/scratchpad/r1245/work/$(basename "$(dirname "$(realpath "$0")")")
rm -rf "$WORK"
mkdir -p "$WORK"
export HOME=$WORK

project() {
  mkdir -p "$WORK/$1/tests"
  printf 'def test_add():\n    assert 1 + 1 == 2, "one and one make two"\n' >"$WORK/$1/tests/test_a.py"
}

run() {
  local dir=$1
  shift
  printf '\n$ cd ~/%s && %s\n' "$dir" "${*//$VENV\/bin\//}"
  (
    cd "$WORK/$dir" && env "$@" | sed "s#$WORK#~#g"
    exit "${PIPESTATUS[0]}"
  )
  printf '[exit %s]\n' "$?"
}

echo "pytest $("$VENV/bin/pytest" --version 2>&1) from $VENV"

project with-project
printf '[project]\nname = "p"\n' >"$WORK/with-project/pyproject.toml"
run with-project "$VENV/bin/pytest" -p no:cacheprovider -v
run with-project/tests "$VENV/bin/pytest" -p no:cacheprovider -v

project only-git
git -C "$WORK/only-git" init -q
run only-git "$VENV/bin/pytest" -p no:cacheprovider -v
run only-git/tests "$VENV/bin/pytest" -p no:cacheprovider -v

project neither
run neither "$VENV/bin/pytest" -p no:cacheprovider -v
run neither/tests "$VENV/bin/pytest" -p no:cacheprovider -v

# Control: a pytest table makes its file the configfile.
project with-pytest
printf '[project]\nname = "p"\n\n[tool.pytest.ini_options]\n' >"$WORK/with-pytest/pyproject.toml"
run with-pytest/tests "$VENV/bin/pytest" -p no:cacheprovider -v
