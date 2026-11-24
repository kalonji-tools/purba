#!/usr/bin/env bash
# Probe pytest's ways to select tests. Each case prints the ids that
# `--collect-only -q` selects, and the exit code.
# Usage: bash run.sh <python with pytest> <empty dir>
set -uo pipefail
py=$1
work=$2
here=$(cd "$(dirname "$0")" && pwd)
cp "$here"/test_math.py "$here"/test_store.py "$here"/pytest.ini "$work"/
cd "$work" || exit 1
"$py" -I -m pytest --version
probe() {
  local name=$1
  shift
  local out
  out=$("$py" -I -m pytest -p no:cacheprovider --collect-only -q "$@" 2>&1)
  local code=$?
  printf '### %s: pytest' "$name"
  printf ' %q' "$@"
  printf '\nexit %s\n' "$code"
  printf '%s\n' "$out" | grep -E '::|^ERROR|^error|no tests|deselected|not found|Wrong expression|^  ' | grep -v '^$'
  echo
}
probe all
probe id-file test_math.py
probe id-test test_math.py::test_parse
probe id-case 'test_math.py::test_parse[a]'
probe id-union test_math.py::test_add test_store.py::test_query_db
probe id-missing test_math.py::test_nope
probe id-prefix test_math.py::test_ad
probe k-substring -k add
probe k-file-name -k math
probe k-mark-name -k slow
probe k-case-id -k 'parse and a'
probe k-repeated -k add -k db
probe m-union -m 'slow or db'
probe m-intersection -m 'slow and db'
probe m-exclusion -m 'slow and not db'
probe m-nesting -m '(slow or db) and not (slow and db)'
probe m-precedence -m 'db or slow and not db'
probe m-repeated -m slow -m db
probe m-and-k -m slow -k 'not migrate'
probe deselect --deselect test_store.py::test_migrate_db
probe ignore --ignore test_store.py
probe m-unknown-mark -m nosuchmark
probe m-syntax-error -m 'slow and'
for n in 200 500 1000; do
  e="$(printf '%*s' "$n" '' | tr ' ' '(')slow$(printf '%*s' "$n" '' | tr ' ' ')')"
  out=$("$py" -I -m pytest -p no:cacheprovider --collect-only -q -m "$e" 2>&1)
  code=$?
  printf '### m-parens-%s: pytest -m <%s parens around slow>\nexit %s\n' "$n" "$n" "$code"
  printf '%s\n' "$out" | grep -E 'RecursionError|collected' | head -2
  echo
done
probe m-unknown-mark-strict --strict-markers -m nosuchmark
probe m-unknown-mark-strict-typo --strict-markers -m slwo
probe path-and-m test_store.py -m slow
probe id-and-deselect test_math.py --deselect test_math.py::test_add_slow
