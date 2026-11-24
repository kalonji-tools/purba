#!/usr/bin/env bash
# Probe unittest's ways to select tests. Each case prints the tests that ran,
# and the exit code. unittest has no tag, so "slow" and "db" are parts of names.
# Usage: bash run.sh <python 3.13> <empty dir>
set -uo pipefail
py=$1
work=$2
here=$(cd "$(dirname "$0")" && pwd)
cp "$here"/test_math.py "$work"/
cd "$work" || exit 1
"$py" -I -c 'import sys; print(sys.version)'
probe() {
  local name=$1
  shift
  local out
  # -I drops the current directory from sys.path; PYTHONPATH is ignored too,
  # so the probe puts the work directory on the path through -c.
  out=$("$py" -I -c 'import sys; sys.path.insert(0, "."); import unittest; unittest.main(module=None, argv=["unittest"] + sys.argv[1:])' -v "$@" 2>&1)
  local code=$?
  printf '### %s: python -m unittest -v' "$name"
  printf ' %q' "$@"
  printf '\nexit %s\n' "$code"
  printf '%s\n' "$out" | grep -E ' \.\.\. |^Ran |NO TESTS|Error|error:'
  echo
}
probe all
probe name-class test_math.Store
probe name-method test_math.Math.test_add
probe name-union test_math.Math.test_add test_math.Store.test_query_db
probe name-missing test_math.Math.test_nope
probe k-substring -k add
probe k-union -k slow -k db
probe k-glob -k '*db_slow'
probe k-class -k Store
probe k-empty -k nosuchname
