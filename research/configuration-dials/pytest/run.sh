#!/usr/bin/env bash
# Probe which pytest dials change a test's outcome, and which level wins.
# usage: run.sh <python with pytest 9.2.0 and pytest-timeout 2.4.0> <empty work dir>
set -u
py=$1
work=$2
here=$(cd "$(dirname "$0")" && pwd)

# case <name> <ini text or -> <env assignment or -> <pytest args...>
case_() {
  local name=$1 ini=$2 envset=$3
  shift 3
  local dir="$work/$name"
  mkdir -p "$dir"
  cp "$here/test_dials.py" "$dir/"
  if [ "$ini" != - ]; then printf '%b' "$ini" >"$dir/pytest.ini"; else printf '[pytest]\n' >"$dir/pytest.ini"; fi
  echo "---- $name"
  [ "$ini" != - ] && {
    echo "pytest.ini:"
    sed 's/^/  /' "$dir/pytest.ini"
  }
  [ "$envset" != - ] && echo "env: $envset"
  echo "args: $*"
  (
    cd "$dir" || exit 99
    if [ "$envset" != - ]; then export "${envset?}"; fi
    "$py" -I -m pytest -p no:cacheprovider -q -rA --no-header "$@" 2>&1 |
      grep -E '^(PASSED|FAILED|ERROR|XFAIL|XPASS|SKIPPED|E +Failed: Timeout|.*Unknown config|.*PytestConfigWarning|.*not found in .markers|.*Tests pass, but|.*passed|.*failed|.*error)' |
      grep -vE '^(  |-)' | head -8
    echo "exit: ${PIPESTATUS[0]}"
  )
}

T='test_dials.py'
# A timeout from a plugin, at the command line, in the environment, and in code.
case_ timeout-none - - "$T" -k "test_sleeps_two_seconds and not marked"
case_ timeout-cli - - "$T" -k "test_sleeps_two_seconds and not marked" --timeout=1
case_ timeout-env - "PYTEST_TIMEOUT=1" "$T" -k "test_sleeps_two_seconds and not marked"
case_ timeout-ini '[pytest]\ntimeout = 1\n' - "$T" -k "test_sleeps_two_seconds and not marked"
case_ timeout-marker-vs-cli - - "$T" -k "marked_five" --timeout=1
case_ timeout-plugin-off '[pytest]\ntimeout = 1\n' - "$T" -k "test_sleeps_two_seconds and not marked" -p no:timeout

# Warnings: the command line, the environment, the project file, and code.
case_ warn-none - - "$T" -k "test_warns_deprecation and not marked"
case_ warn-cli-error - - "$T" -k "test_warns_deprecation and not marked" -W error::DeprecationWarning
case_ warn-env-addopts - "PYTEST_ADDOPTS=-W error::DeprecationWarning" "$T" -k "test_warns_deprecation and not marked"
case_ warn-ini-error '[pytest]\nfilterwarnings =\n    error::DeprecationWarning\n' - "$T" -k "test_warns_deprecation and not marked"
case_ warn-ini-error-cli-ignore '[pytest]\nfilterwarnings =\n    error::DeprecationWarning\n' - "$T" -k "test_warns_deprecation and not marked" -W ignore::DeprecationWarning
case_ warn-ini-error-cli-override '[pytest]\nfilterwarnings =\n    error::DeprecationWarning\n' - "$T" -k "test_warns_deprecation and not marked" -o "filterwarnings=ignore::DeprecationWarning"
case_ warn-ini-error-marker-ignore '[pytest]\nfilterwarnings =\n    error::DeprecationWarning\n' - "$T" -k "marked_ignore"
case_ warn-ini-error-plugin-off '[pytest]\nfilterwarnings =\n    error::DeprecationWarning\n' - "$T" -k "test_warns_deprecation and not marked" -p no:warnings
case_ warn-max-warnings - - "$T" -k "test_warns_deprecation and not marked" --max-warnings=0

# Markers, xfail and the configuration file itself.
case_ marker-default - - "$T" -k "unregistered"
case_ marker-strict - - "$T" -k "unregistered" --strict-markers
case_ xfail-default - - "$T" -k "xfail_that"
case_ xfail-strict - - "$T" -k "xfail_that" -o strict_xfail=true
case_ xfail-runxfail - - "$T" -k "xfail_that" --runxfail
case_ key-misspelled '[pytest]\nfilterwarning = error\n' - "$T" -k "test_warns_deprecation and not marked"
case_ key-misspelled-strict '[pytest]\nfilterwarning = error\n' - "$T" -k "test_warns_deprecation and not marked" --strict-config
