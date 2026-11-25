#!/usr/bin/env bash
# go's main module: the nearest go.mod up from the working folder; a go.work
# above it takes over. With no go.mod, go refuses, and names a .git/config
# that it finds above. A package's id is its import path.
# usage: bash run.sh <empty dir>
set -u
D=$1
export HOME=$D/home GOPATH=$D/gopath GOCACHE=$D/gocache GOMODCACHE=$D/gopath/mod
export GOTOOLCHAIN=local GOPROXY=off GOFLAGS= GOENV=off
unset GOWORK GO111MODULE
mkdir -p "$HOME" "$D/p/a" "$D/none"
echo "== go version"
go version

# show <cwd> [env...] -- <go args...>
show() {
  local cwd=$1
  shift
  local envs=()
  while [ "$1" != "--" ]; do
    envs+=("$1")
    shift
  done
  shift
  echo "-- cwd ${cwd#"$D"/}: ${envs[*]} go $*"
  (
    cd "$cwd" && env "${envs[@]}" go "$@" 2>&1
    echo "exit=$?"
  )
}

printf 'package a\n' >"$D/p/a/a.go"
printf 'package a\n\nimport "testing"\n\nfunc TestLend(t *testing.T) {}\n' >"$D/p/a/a_test.go"
echo "== 1. no go.mod anywhere"
show "$D/p/a" -- env GOMOD
show "$D/p/a" -- test .
echo "== 2. p/.git/config, and no go.mod: go names the folder that holds .git"
git -C "$D/p" init -q
show "$D/p/a" -- test .
echo "== 3. p/go.mod: the import path, from p and from p/a"
printf 'module example.com/m\n\ngo 1.26\n' >"$D/p/go.mod"
show "$D/p" -- env GOMOD
show "$D/p/a" -- env GOMOD
show "$D/p" -- list ./...
show "$D/p/a" -- list ./...
echo "== 4. the name of a test, from p and from p/a"
(cd "$D/p" && go test -v ./... 2>&1 | grep -E '^(=== RUN|ok)' | sed -E -e 's/\t[0-9.]+s$//' -e 's/\t\(cached\)$//')
(cd "$D/p/a" && go test -v ./... 2>&1 | grep -E '^(=== RUN|ok)' | sed -E -e 's/\t[0-9.]+s$//' -e 's/\t\(cached\)$//')
echo "== 5. a go.work above p"
printf 'go 1.26\n\nuse ./p\n' >"$D/go.work"
show "$D/p/a" -- env GOWORK
show "$D/p/a" GOWORK=off -- env GOWORK
echo "== 6. -C p from the folder above p"
show "$D/none" -- -C ../p env GOMOD
