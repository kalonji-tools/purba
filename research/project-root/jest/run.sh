#!/usr/bin/env bash
# Jest's rootDir: the folder of its settings file or of the nearest
# package.json, up from the working folder. With neither, Jest refuses.
# usage: bash run.sh <dir with node_modules/jest@30.5.2> <empty dir>
set -u
NM=$1
D=$2
JEST=$NM/node_modules/.bin/jest
export HOME=$D/home
unset CI
mkdir -p "$HOME" "$D/p/a"
printf 'test("lends", () => { throw new Error("fails on purpose"); });\n' >"$D/p/a/x.test.js"
echo "== jest --version"
"$JEST" --version

# show <cwd> <args...>: rootDir, then the exit code and the FAIL line, which
# names the file as Jest prints it. The test fails on purpose.
show() {
  local cwd=$1
  shift
  echo "-- cwd ${cwd#"$D"/}: jest $*"
  (cd "$cwd" && "$JEST" --ci --showConfig "$@" 2>&1 | grep -E '"rootDir"|Could not find|Error' | sed -E 's/^ +//' | sort -u)
  (
    cd "$cwd" && "$JEST" --ci --verbose "$@" >"$D/run.txt" 2>&1
    echo "exit=$?"
    grep -E '^(PASS|FAIL) ' "$D/run.txt" | sed -E 's/ \([0-9.]+ m?s\)$//'
  )
}

echo "== 1. no package.json and no settings file"
show "$D/p/a"
echo "== 1b. the same, with --rootDir: the walk still refuses"
show "$D/p/a" --rootDir .
echo "== 1c. the same, with the settings as a JSON string: rootDir is the working folder"
show "$D/p/a" --config '{}'
echo "== 2. p/package.json with no \"jest\" key"
printf '{"name": "p"}\n' >"$D/p/package.json"
show "$D/p"
show "$D/p/a"
echo "== 3. --rootDir names p/a, run from p"
show "$D/p" --rootDir a
