#!/usr/bin/env bash
# Go's rules of visibility and of type: an exported name is one that starts with
# an upper-case letter, and a test is a TestXxx with the signature func(*testing.T).
# usage: nix-shell -p go --run "bash run.sh <empty dir>"
set -u
D=$1
cd "$D" || exit 1
export GOFLAGS=-mod=mod GOTOOLCHAIN=local GOPATH="$D/gopath" GOCACHE="$D/gocache"
echo "== go version"
go version

mkdir -p shop/lend app
cat >go.mod <<'EOF'
module example.com/m

go 1.22
EOF
cat >shop/lend/lend.go <<'EOF'
package lend

func Lend() int   { return helper() }
func helper() int { return 1 }
EOF

echo "== 1. another package calls an unexported name"
cat >app/main.go <<'EOF'
package main

import "example.com/m/shop/lend"

func main() { println(lend.Lend(), lend.helper()) }
EOF
go build -o /dev/null ./app 2>&1 | grep -v '^#'

echo "== 2. a Test function with the wrong signature, and one with a lower-case suffix"
cat >shop/lend/lend_test.go <<'EOF'
package lend

import "testing"

func TestLend(t *testing.T)     { if Lend() != 1 { t.Fatal("lend") } }
func TestWrong(n int)           {}
func Testlower(t *testing.T)    {}
func checkHelper(t *testing.T)  {}
EOF
go test ./shop/lend 2>&1 | grep -v '^#' | head -4
echo "== 3. go vet on the same file"
go vet ./shop/lend 2>&1 | grep -v '^#' | head -4
echo "== 4. without TestWrong: which tests run"
sed -i '/TestWrong/d' shop/lend/lend_test.go
go test -v ./shop/lend 2>&1 | grep -v "^#" | head -6
