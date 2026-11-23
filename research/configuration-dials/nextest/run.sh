#!/usr/bin/env bash
# Probe which cargo-nextest dials change a test's outcome, and which level wins.
# usage: run.sh <empty work dir>   (cargo and cargo-nextest on PATH)
set -u
work=$1
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$work/src" "$work/.config"
printf '[package]\nname = "dials"\nversion = "0.0.0"\nedition = "2021"\n\n[workspace]\n' >"$work/Cargo.toml"
cp "$here/lib.rs" "$work/src/lib.rs"
cd "$work" || exit 99
cargo build --tests --offline -q 2>&1 | tail -3

# case <name> <nextest.toml text or -> <env assignment or -> <nextest args...>
case_() {
  local name=$1 cfg=$2 envset=$3
  shift 3
  rm -f .config/nextest.toml
  echo "---- $name"
  if [ "$cfg" != - ]; then
    printf '%b' "$cfg" >.config/nextest.toml
    echo ".config/nextest.toml:"
    sed 's/^/  /' .config/nextest.toml
  fi
  [ "$envset" != - ] && echo "env: $envset"
  echo "args: $*"
  (
    if [ "$envset" != - ]; then export "${envset?}"; fi
    cargo nextest run --offline --no-fail-fast --color never --user-config-file none "$@" 2>&1 |
      grep -E '^\s*(PASS|FAIL|TIMEOUT|FLAKY|TRY|SLOW|SKIP|SIGKILL|SIGTERM|Summary|warning|error)' | grep -v 'Finished' | head -10
    echo "exit: ${PIPESTATUS[0]}"
  )
}

case_ default - - -E 'test(sleeps) | test(first_attempt)'
case_ timeout-config '[profile.default]\nslow-timeout = { period = "1s", terminate-after = 2 }\n' - -E 'test(sleeps)'
case_ timeout-override '[profile.default]\nslow-timeout = { period = "1s", terminate-after = 2 }\n\n[[profile.default.overrides]]\nfilter = "test(sleeps)"\nslow-timeout = { period = "10s", terminate-after = 2 }\n' - -E 'test(sleeps)'
case_ retries-cli - - -E 'test(first_attempt)' --retries 1
case_ retries-env - "NEXTEST_RETRIES=1" -E 'test(first_attempt)'
case_ retries-config '[profile.default]\nretries = 1\n' - -E 'test(first_attempt)'
case_ retries-config-cli-zero '[profile.default]\nretries = 1\n' - -E 'test(first_attempt)' --retries 0
case_ flaky-result-fail - - -E 'test(first_attempt)' --retries 1 --flaky-result fail
case_ run-ignored - - -E 'test(ignored)' --run-ignored all
case_ key-misspelled '[profile.default]\nretires = 1\n' - -E 'test(first_attempt)'
