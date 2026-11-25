#!/usr/bin/env bash
# Probe the claims that decided the scores of criteria 8 and 9, fresh, on the
# port of each version. $1 = a fresh folder to work in; $2 and $3 = the venvs
# of purba's builds A and B, each with pytest and oxitest.
set -uo pipefail
W="$1"
VA="$2"
VB="$3"
K="$(cd "$(dirname "$0")/../benchmark" && pwd)"
rm -rf "$W"
mkdir -p "$W"
run() { # $1 = the folder, then the command
  local at="$1"
  shift
  echo "\$ (cd ${at#"$W"/} && $*)"
  (cd "$at" && env "$@" 2>&1 | sed -e "s#$W#<w>#g" -e "s#$VA#<venv-a>#g" -e "s#$VB#<venv-b>#g" | tail -n 4)
  echo "[exit ${PIPESTATUS[0]}]"
  echo
}
copy() { # $1 = the version, $2 = the copy
  mkdir -p "$2"
  (cd "$K/$1" && tar --exclude=CHECKERS.txt.gz -cf - .) | (cd "$2" && tar -xf -)
}
first() { # $1 = the folder, then the command: only the first line of output
  local at="$1"
  shift
  echo "\$ (cd ${at#"$W"/} && $*)"
  (cd "$at" && env "$@" 2>&1 | sed -e "s#$W#<w>#g" | grep -m1 -E 'root|rootdir')
  echo
}

echo "## 1. Where a run starts: the line each runner prints, from tests/, with neither file"
for v in pytest oxitest purba; do
  copy "$v" "$W/none-$v"
  rm "$W/none-$v/pyproject.toml"
done
first "$W/none-pytest/tests" "$VA/bin/pytest" -p no:cacheprovider
first "$W/none-oxitest/tests" "$VA/bin/oxitest" --color=never
first "$W/none-purba/tests" PURBA_IMPLICIT_ROOT=1 "$VA/bin/purba"
first "$W/none-purba/tests" PURBA_IMPLICIT_ROOT=1 "$VB/bin/purba"

echo "## 2. pytest: from tests/ of the project, rootdir names the project, and an id starts at tests/"
copy pytest "$W/p"
(cd "$W/p/tests" && "$VA/bin/pytest" -p no:cacheprovider -v 2>&1 | grep -E 'rootdir|::test_format' | head -n 2 | sed -e "s#$W#<w>#g")
echo

echo "## 3. Each runner's pattern setting, set to test*.py, a string, and an empty list"
pattern() { # $1 = the version, $2 = the table, $3 = the key, $4 = the value, then the command
  local v="$1" table="$2" key="$3" value="$4"
  shift 4
  local at="$W/pat-$v-$RANDOM"
  copy "$v" "$at"
  mv "$at/tests/test_add_option_format.py" "$at/tests/add_option_format_test.py"
  if grep -q "^\[$table\]" "$at/pyproject.toml"; then
    sed -i "/^\[$table\]/a $key = $value" "$at/pyproject.toml"
  else
    printf '\n[%s]\n%s = %s\n' "$table" "$key" "$value" >>"$at/pyproject.toml"
  fi
  echo "# [$table] $key = $value"
  run "$at" "$@"
}
for value in '["test*.py"]' '"*_test.py"' '[]'; do
  pattern pytest tool.pytest.ini_options python_files "$value" "$VA/bin/pytest" -p no:cacheprovider
  pattern oxitest tool.oxitest python_files "$value" "$VA/bin/oxitest" --color=never
  pattern purba tool.purba test-files "$value" "$VA/bin/purba"
  pattern purba tool.purba test-files "$value" "$VB/bin/purba"
done

echo "## 4. purba: a test whose expected exception is not raised"
copy purba "$W/noraise"
cat >"$W/noraise/tests/test_noraise.py" <<'EOF'
def raised(call):
    try:
        call()
    except Exception as error:
        return error


def test_nothing_raised():
    error = raised(lambda: None)
    assert isinstance(error, TypeError)
EOF
run "$W/noraise" PURBA_IMPLICIT_ROOT=1 "$VA/bin/purba" tests/test_noraise.py
