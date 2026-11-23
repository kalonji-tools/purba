#!/usr/bin/env bash
# Probe which Jest dials change a test's outcome, and which level wins.
# usage: run.sh <dir holding node_modules/jest 30.5.2> <empty work dir>   (node on PATH)
set -u
mods=$1
work=$2
here=$(cd "$(dirname "$0")" && pwd)
cp "$here/dials.test.js" "$work/"
ln -s "$mods/node_modules" "$work/node_modules"
printf '{ "name": "dials" }\n' >"$work/package.json"
cd "$work" || exit 99

# case <name> <jest.config.js text or -> <env assignment or -> <jest args...>
case_() {
  local name=$1 cfg=$2 envset=$3
  shift 3
  rm -f jest.config.js
  rm -rf __snapshots__
  echo "---- $name"
  if [ "$cfg" != - ]; then
    printf '%b' "$cfg" >jest.config.js
    echo "jest.config.js:"
    sed 's/^/  /' jest.config.js
  fi
  [ "$envset" != - ] && echo "env: $envset"
  echo "args: $*"
  (
    unset CI GITHUB_ACTIONS
    if [ "$envset" != - ]; then export "${envset?}"; fi
    ./node_modules/.bin/jest --no-color --verbose --no-cache "$@" 2>&1 |
      grep -E '(✓|✕|○|Exceeded timeout|Tests:|Snapshots:|Unknown option|not written|obsolete|retry)' | head -8
    echo "exit: ${PIPESTATUS[0]}"
  )
}

case_ timeout-default - - -t "waits one second$"
case_ timeout-cli - - -t "waits one second$" --testTimeout=500
case_ timeout-config 'module.exports = { testTimeout: 500 };\n' - -t "waits one second$"
case_ timeout-code-vs-cli - - -t "marked five" --testTimeout=500
case_ snapshot-local - - -t "snapshot"
case_ snapshot-ci-env - "CI=true" -t "snapshot"
case_ snapshot-ci-flag - - -t "snapshot" --ci
case_ key-misspelled 'module.exports = { testTmeout: 500 };\n' - -t "waits one second$"
