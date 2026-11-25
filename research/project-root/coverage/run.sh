#!/usr/bin/env bash
# coverage.py reads its settings file from the working folder only, and names
# each file of a report relative to the working folder.
# usage: bash run.sh <coverage 7.17.5> <empty dir>
set -u
COV=$1
D=$2
export HOME=$D/home XDG_CONFIG_HOME=$D/home/.config
unset COVERAGE_RCFILE
mkdir -p "$HOME" "$D/p/a"
printf 'x = 1\n' >"$D/p/a/m.py"
printf '[run]\nbranch = true\n' >"$D/p/.coveragerc"
echo "== coverage --version"
"$COV" --version | head -1

# read <cwd> [env...]: the settings files that coverage read
read_cfg() {
  local cwd=$1
  shift
  echo "-- cwd ${cwd#"$D"/}: $* coverage debug config"
  (cd "$cwd" && env "$@" "$COV" debug config 2>&1 | grep -E '^ *(config_files_read|branch):' | sed -E 's/^ +//')
}

echo "== 1. p/.coveragerc, read from p and from p/a"
read_cfg "$D/p"
read_cfg "$D/p/a"
echo "== 2. COVERAGE_RCFILE names it from p/a"
read_cfg "$D/p/a" COVERAGE_RCFILE="$D/p/.coveragerc"

echo "== 3. the name of the file in a report, from p and from p/a"
(cd "$D/p" && "$COV" run --data-file="$D/d1" a/m.py && "$COV" report --data-file="$D/d1" 2>&1 | grep 'm.py' | sed 's/^/cwd p: /')
(cd "$D/p/a" && "$COV" run --data-file="$D/d2" m.py && "$COV" report --data-file="$D/d2" 2>&1 | grep 'm.py' | sed 's/^/cwd p\/a: /')

echo "== 4. the flag that names the file"
"$COV" report --help | grep -E -- "--rcfile"
