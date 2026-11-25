#!/usr/bin/env bash
VENV=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/96b6d852-95a0-47f0-872d-fb178e3ab17c/scratchpad/r1245/venv-a
# Each claim of rules-*/TESTS.md, on wheel A: what a test file is, what a test
# is, what an expectation is, the command, the lines and the exit codes.
set -u
WORK=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/96b6d852-95a0-47f0-872d-fb178e3ab17c/scratchpad/r1245/work/$(basename "$(dirname "$(realpath "$0")")")
rm -rf "$WORK"
mkdir -p "$WORK"
export HOME=$WORK
unset PURBA_IMPLICIT_ROOT

# file <path under ~/p> : writes stdin to it and prints it
file() {
  mkdir -p "$(dirname "$WORK/p/$1")"
  cat >"$WORK/p/$1"
  printf '\n# ~/p/%s\n' "$1"
  cat "$WORK/p/$1"
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

echo "purba $("$VENV/bin/python" -c 'import importlib.metadata as m; print(m.version("purba"))') from $VENV"

git -C "$WORK" init -q p
printf '# ~/p is a git repository\n'
file pyproject.toml <<'EOF'
[project]
name = "p"

[tool.purba]
EOF
file .gitignore <<'EOF'
build/
EOF

# Which files are test files.
for name in tests/test_names.py tests/names_test.py tests/test.py tests/tests.py \
  tests/.hidden/test_hidden.py build/test_built.py tests/deep/er/test_deep.py; do
  printf 'def test_one():\n    assert 1 == 1, "one is one"\n' | file "$name"
done
run p "$VENV/bin/purba"
run p "$VENV/bin/purba" tests/names_test.py
run p "$VENV/bin/purba" tests/.hidden/test_hidden.py
run p "$VENV/bin/purba" build/test_built.py
run p "$VENV/bin/purba" tests/deep
run p/tests/deep "$VENV/bin/purba"
run p/tests "$VENV/bin/purba" deep
run p/tests "$VENV/bin/purba" tests/deep
rm -r "$WORK/p/tests" "$WORK/p/build"

# What a test is.
file tests/test_kinds.py <<'EOF'
import functools


def helper():
    return 2


def testplain():
    assert helper() == 2, "the helper gives two"


def test_param(writer):
    assert writer == 1, "never runs"


@functools.cache
def test_decorated():
    assert 1 == 1, "never runs"


class TestGroup:
    def test_method(self):
        assert 1 == 1, "never runs"
EOF
run p "$VENV/bin/purba" tests/test_kinds.py

# What an expectation is.
file tests/test_expect.py <<'EOF'
def total(items):
    if not items:
        return 0
    return sum(items)


def test_passes():
    lines = "a\nb".split("\n")
    assert len(lines) == 2, "two lines"
    assert lines[0] == "a", "the first is a"


def test_fails():
    assert "a".upper() == "b", "upper of a"


def test_raises():
    {}["missing"]
    assert 1 == 1, "never reached"


def test_assert_then_code():
    x = 1
    assert x == 1, "x is one"
    x = 2
    assert x == 2, "x is two"


def test_computed():
    for expected in [1, 2]:
        pass
    assert 1 == expected, "computed"


def test_one_comparison():
    ready = True
    assert ready, "ready"


def test_four_forms():
    name = "test_a.py"
    assert not name.isupper(), "lower"
    assert isinstance(name, str), "a string"
    assert name.startswith("test_"), "starts test_"
    assert name.endswith(".py"), "ends .py"


def test_all_any():
    sizes = [1, 2, 3]
    assert all(size > 0 for size in sizes), "each is positive"
    assert any(size == 2 for size in sizes), "one is two"


def test_with():
    with open(__file__) as handle:
        text = handle.read()
        assert text.startswith("def"), "starts def"


def test_second_path():
    assert total([1, 2]) == 3, "the sum"


def test_none():
    x = 1
EOF
run p "$VENV/bin/purba" tests/test_expect.py

# A private name.
file tests/test_private.py <<'EOF'
import json.decoder


def test_private():
    table = json.decoder._CONSTANTS
    assert len(table) == 3, "three constants"
EOF
run p "$VENV/bin/purba" tests/test_private.py

# The command.
run p "$VENV/bin/purba" --help
run p "$VENV/bin/purba" tests/test_expect.py tests/test_kinds.py
run p "$VENV/bin/purba" tests/missing.py

# The exit codes of a raise alone, a branch in the body, and no case.
file tests/test_more.py <<'PY'
def test_raises_alone():
    {}["missing"]
    assert 1 == 1, "never reached"


def test_branch():
    x = 1
    if x:
        y = 2
    else:
        y = 3
    assert y == 2, "y is two"
PY
run p "$VENV/bin/purba" tests/test_more.py
rm -r "$WORK/p/tests"
mkdir "$WORK/p/tests"
printf '\n# ~/p/tests is now empty\n'
run p "$VENV/bin/purba"
