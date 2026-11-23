#!/usr/bin/env bash
# Probe which `go test` dials change a test's outcome, and which level wins.
# usage: run.sh <empty work dir>   (go on PATH)
set -u
work=$1
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$work/mod" "$work/home"
cp "$here/dials_test.go" "$work/mod/"
printf 'module example.com/dials\n\ngo 1.26\n' >"$work/mod/go.mod"
export HOME="$work/home" GOPATH="$work/home/gopath" GOCACHE="$work/home/cache" GOENV="$work/home/goenv" GOTOOLCHAIN=local GOFLAGS=
cd "$work/mod" || exit 99

# case <name> <env assignment or -> <go test args...>
case_() {
  local name=$1 envset=$2
  shift 2
  echo "---- $name"
  [ "$envset" != - ] && echo "env: $envset"
  echo "args: $*"
  (
    if [ "$envset" != - ]; then export "${envset?}"; fi
    go test -count=1 "$@" . 2>&1 | grep -E '^(--- |ok|FAIL|panic:|\./dials_test|# |.*fmt.Sprintf)' | head -6
    echo "exit: ${PIPESTATUS[0]}"
  )
}

case_ vet-default - -run 'TestSleeps'
case_ vet-off - -vet=off -run 'TestSleeps'
case_ timeout-cli - -vet=off -run 'TestSleeps' -timeout=1s
case_ timeout-goflags "GOFLAGS=-timeout=1s" -vet=off -run 'TestSleeps'
case_ timeout-goflags-cli-wins "GOFLAGS=-timeout=1s" -vet=off -run 'TestSleeps' -timeout=10s
case_ short-off - -vet=off -run 'TestLongUnlessShort'
case_ short-on - -vet=off -run 'TestLongUnlessShort' -short
# The user's file: `go env -w` writes GOENV, which a later run reads.
go env -w GOFLAGS=-timeout=1s
echo "---- goenv-file"
echo "GOENV file: $(cat "$GOENV")"
(
  unset GOFLAGS
  go test -count=1 -vet=off -run TestSleeps . 2>&1 | grep -E '^(--- |ok|FAIL|panic:)' | head -4
  echo "exit: ${PIPESTATUS[0]}"
)
