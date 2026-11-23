#!/usr/bin/env bash
# Probe which Vitest dials change a test's outcome, and which level wins.
# usage: run.sh <dir holding node_modules/vitest 5.1.1> <empty work dir>   (node on PATH)
set -u
mods=$1
work=$2
here=$(cd "$(dirname "$0")" && pwd)
cp "$here"/*.test.mjs "$work/"
ln -s "$mods/node_modules" "$work/node_modules"
printf '{ "type": "module" }\n' >"$work/package.json"
cd "$work" || exit 99

# case <name> <vitest.config.mjs text or -> <env assignment or -> <vitest args...>
case_() {
  local name=$1 cfg=$2 envset=$3
  shift 3
  rm -f vitest.config.mjs
  echo "---- $name"
  if [ "$cfg" != - ]; then
    printf '%b' "$cfg" >vitest.config.mjs
    echo "vitest.config.mjs:"
    sed 's/^/  /' vitest.config.mjs
  fi
  [ "$envset" != - ] && echo "env: $envset"
  echo "args: $*"
  (
    unset CI GITHUB_ACTIONS
    if [ "$envset" != - ]; then export "${envset?}"; fi
    ./node_modules/.bin/vitest run --no-color --reporter=verbose "$@" 2>&1 |
      grep -E '(✓|×|↓|Error:|Tests |Test Files|Unknown|unknown|only)' | grep -v '^ *❯' | head -8
    echo "exit: ${PIPESTATUS[0]}"
  )
}

case_ timeout-default - - dials.test.mjs -t "waits one second$"
case_ timeout-cli - - dials.test.mjs -t "waits one second$" --testTimeout=500
case_ timeout-config 'export default { test: { testTimeout: 500 } };\n' - dials.test.mjs -t "waits one second$"
case_ timeout-code-vs-cli - - dials.test.mjs -t "marked five" --testTimeout=500
case_ retry-none - - dials.test.mjs -t "first attempt"
case_ retry-cli - - dials.test.mjs -t "first attempt" --retry=1
case_ only-local - - only.test.mjs
case_ only-ci - "CI=true" only.test.mjs
case_ only-ci-allow - "CI=true" only.test.mjs --allowOnly
case_ key-misspelled 'export default { test: { testTmeout: 500 } };\n' - dials.test.mjs -t "waits one second$"
