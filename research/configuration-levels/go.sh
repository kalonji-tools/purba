#!/usr/bin/env bash
# editorconfig-checker-disable-file: its heredocs hold Go, which gofmt indents with tabs.
# Probe: go test's levels: $GOROOT/go.env, the user's go env file, the process environment,
# GOFLAGS, the command line, go.mod, and //go:debug in code.
# Usage: go.sh <empty-work-dir>   (run under `nix-shell -p go --run`)
set -u
W=$1
mkdir -p "$W"
cd "$W" || exit 1
export HOME="$W/home" XDG_CONFIG_HOME="$W/home/.config" GOPATH="$W/gopath" GOCACHE="$W/gocache"
export GOTOOLCHAIN=local GOPROXY=off GOFLAGS= GODEBUG=
unset GOENV
mkdir -p "$HOME" "$XDG_CONFIG_HOME"
t() {
  echo "\$ (cd ${PWD#$W/}) ${GENV:-}go test $*"
  env ${GENV:-} go test "$@" 2>&1 | grep -E '^(ok|FAIL|---|panic: test timed out)|recovered' | sed 's/\t[0-9.]*s$//; s/(cached)//' | head -3
  echo
}

echo "## 0. the system level: \$GOROOT/go.env (read-only here, so shown, not changed)"
echo "\$ cat \$(go env GOROOT)/go.env | grep -v '^#'"
grep -v '^#' "$(go env GOROOT)/go.env" | grep -v '^$'
echo

mkdir -p mod/sub
cat >mod/go.mod <<'EOF'
module example.invalid/probe

go 1.26
EOF
cat >mod/sleep_test.go <<'EOF'
package probe

import (
	"testing"
	"time"
)

func TestSleep(t *testing.T) { time.Sleep(2 * time.Second) }
EOF

echo "## 1. -timeout: the user's go env file, then GOFLAGS in the environment, then the command line"
(cd mod && t -count=1 .)
(cd mod && go env -w GOFLAGS=-timeout=1s && echo "\$ go env -w GOFLAGS=-timeout=1s  (writes $(go env GOENV | sed "s#$W/##"))")
(cd mod && t -count=1 .)
(cd mod && GENV="GOFLAGS=-timeout=5s " t -count=1 .)
(cd mod && GENV="GOFLAGS=-timeout=5s " t -count=1 -timeout=1s .)
echo "   GOENV=off drops the user's file"
(cd mod && GENV="GOENV=off " t -count=1 .)
(cd mod && go env -u GOFLAGS)

echo "## 2. GODEBUG: go.mod's godebug line, then //go:debug in a test file, then the GODEBUG variable"
rm mod/sleep_test.go
cat >mod/nil_test.go <<'EOF'
package probe

import "testing"

func TestPanicNil(t *testing.T) {
	defer func() { t.Logf("recovered %T", recover()) }()
	panic(nil)
}
EOF
(cd mod && t -count=1 -v .)
printf 'module example.invalid/probe\n\ngo 1.26\n\ngodebug panicnil=1\n' >mod/go.mod
(cd mod && t -count=1 -v .)
sed -i '1i //go:debug panicnil=0' mod/nil_test.go
(cd mod && t -count=1 -v .)
(cd mod && GENV="GODEBUG=panicnil=1 " t -count=1 -v .)

echo "## 3. go.mod is found by walking up from the cwd: sub/ has no //go:debug, so the parent go.mod's panicnil=1 applies"
sed '1d' mod/nil_test.go >mod/sub/nil_test.go
(cd mod/sub && t -count=1 -v .)
