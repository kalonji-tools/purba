#!/usr/bin/env bash
# What go test picks as a test file and as a test, and what go build leaves out.
# usage: nix-shell -p go --run "bash run.sh <empty dir>"
set -u
D=$1
cd "$D" || exit 1
export GOFLAGS= GOPATH="$D/gopath" GOCACHE="$D/gocache" GOTOOLCHAIN=local GO111MODULE=on
mkdir -p shop/testdata shop/_old
cd shop || exit 1
cat >go.mod <<'EOF'
module example.com/shop

go 1.22
EOF
cat >shop.go <<'EOF'
package shop

func Make() int { return 1 }
EOF
cat >shop_test.go <<'EOF'
package shop

import "testing"

func TestLend(t *testing.T) { check(t, Make(), 1) }

func Testlower(t *testing.T) {} // lowercase after Test: not a test

func check(t *testing.T, got, want int) { // a helper beside the tests
	t.Helper()
	if got != want {
		t.Fatalf("got %d, want %d", got, want)
	}
}
EOF
cat >helpers_test.go <<'EOF'
package shop

func Two() int { return 2 } // in a _test.go file: only the test build sees it
EOF
echo 'package broken; this is not Go' >testdata/broken.go
echo 'package broken; this is not Go' >_old/broken.go

echo "== go version"
go version
echo "== go list: GoFiles and TestGoFiles"
go list -f 'GoFiles={{.GoFiles}} TestGoFiles={{.TestGoFiles}}' ./...
echo "== go test -v ./..."
go test -v -vet=off ./... 2>&1
echo "== go vet ./..."
go vet ./... 2>&1
echo "== a non-test file that calls a helper of a _test.go file"
echo 'package shop; func UseTwo() int { return Two() }' >use.go
go build ./... 2>&1 | head -3
