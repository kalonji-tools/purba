#!/usr/bin/env bash
# Probe the ways `go test` selects tests. Each case prints the tests that ran,
# and the exit code.
# Usage: nix-shell -p go --run "bash run.sh <empty dir>"
set -uo pipefail
work=$1
here=$(cd "$(dirname "$0")" && pwd)
cp "$here"/sel_test.go "$here"/tagged_test.go "$work"/
cd "$work" || exit 1
printf 'module sel\n\ngo 1.26\n' >go.mod
export GOFLAGS=-count=1
go version
probe() {
  local name=$1
  shift
  local out
  out=$(go test -v "$@" 2>&1)
  local code=$?
  printf '### %s: go test -v' "$name"
  printf ' %q' "$@"
  printf '\nexit %s\n' "$code"
  printf '%s\n' "$out" | grep -E '^=== RUN|warning|^FAIL|error|invalid'
  echo
}
probe all
probe run-substring -run Add
probe run-anchored -run '^TestAdd$'
probe run-union -run 'Slow|DB'
probe run-intersection -run 'DB.*Slow|Slow.*DB'
probe run-exclusion -run Slow -skip DB
probe run-repeated -run Slow -run DB
probe skip-repeated -skip DB -skip Slow
probe run-subtest -run 'TestParse/^a$'
probe run-empty -run Nope
probe run-bad-regex -run '('
probe tags-slow -tags slow
probe tags-slow-nodb -tags slow,nodb
