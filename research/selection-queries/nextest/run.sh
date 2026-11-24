#!/usr/bin/env bash
# Probe the ways cargo-nextest and cargo test select tests. Each case prints
# the tests that `list` selects, and the exit code.
# Usage: bash run.sh <empty dir>
set -uo pipefail
work=$1
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$work/src"
cp "$here/lib.rs" "$work/src/lib.rs"
printf '[package]\nname = "sel"\nversion = "0.1.0"\nedition = "2021"\n\n[workspace]\n' >"$work/Cargo.toml"
cd "$work" || exit 1
cargo --version
cargo nextest --version | head -1
cargo build --tests -q 2>&1
nx() {
  local name=$1
  shift
  local out
  out=$(cargo nextest list --message-format oneline "$@" 2>&1)
  local code=$?
  printf '### %s: cargo nextest list' "$name"
  printf ' %q' "$@"
  printf '\nexit %s\n' "$code"
  printf '%s\n' "$out" | grep -E '^sel |error|no tests'
  echo
}
ct() {
  local name=$1
  shift
  local out
  out=$(cargo test -q --lib -- --list "$@" 2>&1)
  local code=$?
  printf '### %s: cargo test --lib -- --list' "$name"
  printf ' %q' "$@"
  printf '\nexit %s\n' "$code"
  printf '%s\n' "$out" | grep -E ': test$|error'
  echo
}
nx all
nx substring add
nx substring-union slow db
nx skip -- --skip db
nx exact -- --exact math::add
nx e-union -E 'test(slow) | test(db)'
nx e-intersection -E 'test(slow) & test(db)'
nx e-exclusion -E 'test(slow) - test(db)'
nx e-nesting -E '(test(slow) | test(db)) - (test(slow) & test(db))'
nx e-precedence -E 'test(db) | test(slow) & not test(db)'
nx e-repeated -E 'test(slow)' -E 'test(db)'
nx e-and-substring -E 'test(slow)' db
nx e-module-prefix -E 'test(/^store::/)'
nx e-empty -E 'test(nosuchname)'
nx e-syntax-error -E 'test(slow) &'
nx e-not-paren -E 'not(test(db))'
out=$(cargo nextest run -E 'test(nosuchname)' 2>&1)
code=$?
printf '### e-empty-run: cargo nextest run -E %q\nexit %s\n' 'test(nosuchname)' "$code"
printf '%s\n' "$out" | grep -E 'error|no tests'
echo
ct all
ct substring add
ct substring-union slow db
ct skip --skip db
ct skip-repeated slow --skip db --skip add
ct exact --exact math::add
ct empty nosuchname
ct slow-not-db slow --skip db
out=$(cargo test -q --lib nosuchname 2>&1)
code=$?
printf '### empty-run: cargo test --lib nosuchname\nexit %s\n' "$code"
printf '%s\n' "$out" | grep -E 'test result'
echo
for n in 1000 10000; do
  e="$(printf '%*s' "$n" '' | tr ' ' '(')test(slow)$(printf '%*s' "$n" '' | tr ' ' ')')"
  out=$(cargo nextest list --message-format oneline -E "$e" 2>&1)
  code=$?
  printf '### e-parens-%s: cargo nextest list -E <%s parens around test(slow)>\nexit %s\n' "$n" "$n" "$code"
  printf '%s\n' "$out" | grep -E '^sel |overflow' | head -2
  echo
done
