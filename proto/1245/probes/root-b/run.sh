#!/usr/bin/env bash
VENV=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/96b6d852-95a0-47f0-872d-fb178e3ab17c/scratchpad/r1245/venv-b
# The implicit root: each step of the ladder, the refusal, and a bad value.
# HOME is the work folder, so purba shows each root as ~/<project>.
set -u
WORK=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/96b6d852-95a0-47f0-872d-fb178e3ab17c/scratchpad/r1245/work/$(basename "$(dirname "$(realpath "$0")")")
rm -rf "$WORK"
mkdir -p "$WORK"
export HOME=$WORK
unset PURBA_IMPLICIT_ROOT

project() {
  mkdir -p "$WORK/$1/tests"
  printf 'def test_add():\n    assert 1 + 1 == 2, "one and one make two"\n' >"$WORK/$1/tests/test_a.py"
}

run() {
  local dir=$1
  shift
  printf '\n$ cd ~/%s && %s\n' "$dir" "${*//$VENV\/bin\//}"
  (cd "$WORK/$dir" && env "$@")
  printf '[exit %s]\n' "$?"
}

echo "purba $("$VENV/bin/python" -c 'import importlib.metadata as m; print(m.version("purba"))') from $VENV"

# Step 2: a pyproject.toml with [project], and no [tool.purba].
project with-project
printf '[project]\nname = "p"\n' >"$WORK/with-project/pyproject.toml"
run with-project "$VENV/bin/purba"
run with-project PURBA_IMPLICIT_ROOT=0 "$VENV/bin/purba"
run with-project PURBA_IMPLICIT_ROOT= "$VENV/bin/purba"
run with-project PURBA_IMPLICIT_ROOT=yes "$VENV/bin/purba"
run with-project PURBA_IMPLICIT_ROOT=1 "$VENV/bin/purba"
run with-project/tests PURBA_IMPLICIT_ROOT=1 "$VENV/bin/purba"

# Step 3: only .git.
project only-git
git -C "$WORK/only-git" init -q
run only-git "$VENV/bin/purba"
run only-git PURBA_IMPLICIT_ROOT=1 "$VENV/bin/purba"
run only-git/tests PURBA_IMPLICIT_ROOT=1 "$VENV/bin/purba"

# Step 4: neither; the root and the ids follow the working folder.
project neither
run neither "$VENV/bin/purba"
run neither PURBA_IMPLICIT_ROOT=1 "$VENV/bin/purba"
run neither/tests PURBA_IMPLICIT_ROOT=1 "$VENV/bin/purba"

# Control: [tool.purba] prints no root line.
project explicit
printf '[project]\nname = "p"\n\n[tool.purba]\n' >"$WORK/explicit/pyproject.toml"
run explicit PURBA_IMPLICIT_ROOT=1 "$VENV/bin/purba"
