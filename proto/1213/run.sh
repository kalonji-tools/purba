#!/usr/bin/env bash
# Prototype harness for #1213. This branch is never merged.
#
#   proto/1213/run.sh <purba> <out-dir> <work-dir> [baseline]
#
# For each scenario, it makes a fresh project under <work-dir>, a fake HOME and
# XDG_CONFIG_HOME, and a scratch system file, then runs the installed <purba>
# once per variant. Each run lands in <out-dir>/<scenario>.<variant>.txt.
# In `baseline` mode, each setup runs once, with no switch of the prototype.
set -euo pipefail

purba=$1
out=$2
work=$3
mode=${4:-proto}
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out" "$work"

RULES=(higher-overwrites code-wins stricter-wins)

# The module of every timeout scenario: one test sleeps 2 s, one is quick.
slow_module() {
  cat >"$1" <<'EOF'
import time


def test_slow():
    time.sleep(2)
    assert 1 + 1 == 2


def test_quick():
    assert 1 + 1 == 2
EOF
}

# The same module, with a timeout of 6 s set in code on test_slow.
slow_module_code() {
  cat >"$1" <<'EOF'
import time


# purba: set[timeout] 6
def test_slow():
    time.sleep(2)
    assert 1 + 1 == 2


def test_quick():
    assert 1 + 1 == 2
EOF
}

# A module that holds one suppression, which its test uses.
suppressed_module() {
  cat >"$1" <<'EOF'
import json


# purba: suppress[computed-expected] the payload is its own expected value
def test_round_trip():
    obj = {"a": 1}
    assert json.loads(json.dumps(obj)) == obj
EOF
}

# A module whose test sets suppression-cap in code.
cap_in_code_module() {
  cat >"$1" <<'EOF'
import json


# purba: set[suppression-cap] 5
# purba: suppress[computed-expected] the payload is its own expected value
def test_round_trip():
    obj = {"a": 1}
    assert json.loads(json.dumps(obj)) == obj
EOF
}

# setup <dir>: a fresh tree with proj/, home/, xdg/ and system/.
setup() {
  rm -rf "$1"
  mkdir -p "$1/proj" "$1/home" "$1/xdg/purba" "$1/system"
}

# go <scenario> <variant> <dir> <module> <env...> -- <flags...>
# Runs purba in <dir>/proj on <module>, and writes the run file.
go() {
  local scenario=$1 variant=$2 dir=$3 module=$4
  shift 4
  local -a vars=() flags=()
  while (($#)) && [[ $1 != -- ]]; do
    vars+=("$1")
    shift
  done
  [[ ${1:-} == -- ]] && shift
  flags=("$@")
  local file="$out/$scenario.$variant.txt"
  local -a base=(
    "PATH=/run/current-system/sw/bin:/usr/bin:/bin"
    "HOME=$dir/home"
    "XDG_CONFIG_HOME=$dir/xdg"
    "PURBA_PROTO_SYSTEM=$dir/system/purba.toml"
  )
  local start end status
  start=$(date +%s.%N)
  set +e
  (cd "$dir/proj" && env -i "${base[@]}" "${vars[@]}" timeout 60 "$purba" "${flags[@]}" "$module" \
    >"$dir/stdout" 2>"$dir/stderr")
  status=$?
  set -e
  end=$(date +%s.%N)
  {
    echo "scenario: $scenario"
    echo "variant: $variant"
    echo "purba: ${LABEL:-$purba}"
    echo "env: ${vars[*]:-(none)}"
    echo "command: cd <tmp>/proj && purba ${flags[*]:+${flags[*]} }$module"
    for config in proj/pyproject.toml xdg/purba/purba.toml system/purba.toml; do
      if [[ -f $dir/$config ]]; then
        echo "--- file <tmp>/$config"
        cat "$dir/$config"
      fi
    done
    echo "--- module <tmp>/proj/$module"
    cat "$dir/proj/$module"
    echo "--- stdout"
    cat "$dir/stdout"
    echo "--- stderr"
    cat "$dir/stderr"
    echo "--- exit $status"
    awk -v a="$start" -v b="$end" 'BEGIN { printf "--- seconds %.1f\n", b - a }'
  } | sed "s#$dir#<tmp>#g" >"$file"
}

# each <scenario> <setup-fn> <module> <extra variants spec> -- <flags...>
# Runs the scenario once per rule, or once in baseline mode. `$switches` names
# extra switches for the variant name, e.g. "FENCE=fenced".
variants() {
  local scenario=$1 builder=$2 module=$3 extra=$4
  shift 4
  local dir="$work/$scenario"
  if [[ $mode == baseline ]]; then
    setup "$dir"
    "$builder" "$dir"
    go "$scenario" baseline "$dir" "$module" "${env_extra[@]}" "$@"
    return
  fi
  local rule
  for rule in "${RULES[@]}"; do
    setup "$dir"
    "$builder" "$dir"
    local -a vars=("PURBA_PROTO_RULE=$rule")
    local name=$rule
    local pair
    for pair in $extra; do
      vars+=("PURBA_PROTO_$pair")
      name+=".${pair#*=}"
    done
    go "$scenario" "$name" "$dir" "$module" "${vars[@]}" "${env_extra[@]}" "$@"
  done
}

env_extra=()

# S0: no configuration at all.
s0() { slow_module "$1/proj/test_mod.py"; }
variants S0 s0 test_mod.py "" --

# S1: the project caps suppressions at 0, the module holds 1, the command line
# passes 50.
s1() {
  suppressed_module "$1/proj/test_mod.py"
  printf '[tool.purba]\nsuppression-cap = 0\n' >"$1/proj/pyproject.toml"
}
variants S1 s1 test_mod.py "" -- --suppression-cap 50
s1_bare() { s1 "$1"; }
variants S1-no-flag s1_bare test_mod.py "" --

# S2: a user file sets timeout = 1, the project file has no [tool.purba].
s2_dev() {
  slow_module "$1/proj/test_mod.py"
  printf '[project]\nname = "demo"\n' >"$1/proj/pyproject.toml"
  printf 'timeout = 1\n' >"$1/xdg/purba/purba.toml"
}
s2_ci() {
  slow_module "$1/proj/test_mod.py"
  printf '[project]\nname = "demo"\n' >"$1/proj/pyproject.toml"
}
s2_project() {
  slow_module "$1/proj/test_mod.py"
  printf '[tool.purba]\ntimeout = 10\n' >"$1/proj/pyproject.toml"
  printf 'timeout = 1\n' >"$1/xdg/purba/purba.toml"
}
if [[ $mode == baseline ]]; then
  variants S2-dev s2_dev test_mod.py "" --
  variants S2-ci s2_ci test_mod.py "" --
  variants S2-project10 s2_project test_mod.py "" --
else
  for user in merge drop; do
    variants S2-dev s2_dev test_mod.py "USER=$user" --
    variants S2-project10 s2_project test_mod.py "USER=$user" --
  done
  variants S2-ci s2_ci test_mod.py "" --
fi

# S3: project 4 s, code 6 s, command line 1 s, the test sleeps 2 s.
s3() {
  slow_module_code "$1/proj/test_mod.py"
  printf '[tool.purba]\ntimeout = 4\n' >"$1/proj/pyproject.toml"
}
variants S3 s3 test_mod.py "" -- --timeout 1
s3_bare() { s3 "$1"; }
variants S3-no-flag s3_bare test_mod.py "" --

# S4: timeout in the user file (5) and in the system file (1).
s4() {
  slow_module "$1/proj/test_mod.py"
  printf 'timeout = 5\n' >"$1/xdg/purba/purba.toml"
  printf 'timeout = 1\n' >"$1/system/purba.toml"
}
if [[ $mode == baseline ]]; then
  variants S4 s4 test_mod.py "" --
else
  for fence in open fenced; do
    variants S4 s4 test_mod.py "FENCE=$fence" --
  done
fi

# S5: a misspelled key in the project file.
s5() {
  slow_module "$1/proj/test_mod.py"
  # A misspelling that the typos hook does not know, so the hook leaves it.
  printf '[tool.purba]\ntmeout = 5\n' >"$1/proj/pyproject.toml"
}
variants S5 s5 test_mod.py "" --

# S6: PURBA_TIMEOUT=1 beside a project timeout of 10.
s6() {
  slow_module "$1/proj/test_mod.py"
  printf '[tool.purba]\ntimeout = 10\n' >"$1/proj/pyproject.toml"
}
env_extra=(PURBA_TIMEOUT=1)
variants S6 s6 test_mod.py "" --
env_extra=()

# S7: retries = 2 in the project file.
s7() {
  slow_module "$1/proj/test_mod.py"
  printf '[tool.purba]\nretries = 2\n' >"$1/proj/pyproject.toml"
}
if [[ $mode == baseline ]]; then
  variants S7 s7 test_mod.py "" --
else
  for fence in open fenced; do
    variants S7 s7 test_mod.py "FENCE=$fence" --
  done
fi

# S8: workers = 4 in the user file.
s8() {
  slow_module "$1/proj/test_mod.py"
  printf 'workers = 4\n' >"$1/xdg/purba/purba.toml"
}
if [[ $mode == baseline ]]; then
  variants S8 s8 test_mod.py "" --
else
  for fence in open fenced; do
    variants S8 s8 test_mod.py "FENCE=$fence" --
  done
fi

# S9: a user file sets timeout = 0, the project sets timeout = 1.
s9() {
  slow_module "$1/proj/test_mod.py"
  printf '[tool.purba]\ntimeout = 1\n' >"$1/proj/pyproject.toml"
  printf 'timeout = 0\n' >"$1/xdg/purba/purba.toml"
}
variants S9 s9 test_mod.py "" --

# S10: code sets suppression-cap.
s10() { cap_in_code_module "$1/proj/test_mod.py"; }
if [[ $mode == baseline ]]; then
  variants S10 s10 test_mod.py "" --
else
  for fence in open fenced; do
    variants S10 s10 test_mod.py "FENCE=$fence" --
  done
fi

echo "runs in $out ($here)"
