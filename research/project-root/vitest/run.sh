#!/usr/bin/env bash
# Vitest's root: the working folder, or --root. It does not walk up, and it
# runs with no settings file and no package.json.
# usage: bash run.sh <dir with node_modules/vitest@5.1.1> <empty dir>
set -u
NM=$1
D=$2
VITEST=$NM/node_modules/.bin/vitest
export HOME=$D/home
unset CI
mkdir -p "$HOME" "$D/p/a"
printf 'test("lends", () => {});\n' >"$D/p/a/x.test.js"
echo "== vitest --version"
"$VITEST" --version

# show <cwd> <args...>: each test id that vitest lists, and the exit code
show() {
  local cwd=$1
  shift
  echo "-- cwd ${cwd#"$D"/}: vitest list --globals $*"
  (
    cd "$cwd" && "$VITEST" list --globals "$@" 2>&1 | grep -i -E 'lends|error|no test|include'
    echo "exit=${PIPESTATUS[0]}"
  )
}

echo "== 1. no package.json and no settings file"
show "$D/p"
show "$D/p/a"
echo "== 2. p/package.json and p/vitest.config.mjs, run from p/a: not read"
printf '{"name": "p"}\n' >"$D/p/package.json"
printf 'export default { test: { name: "fromconfig" } };\n' >"$D/p/vitest.config.mjs"
show "$D/p/a"
echo "== 3. --root .. from p/a: the settings file now applies, and names the project"
show "$D/p/a" --root ..
