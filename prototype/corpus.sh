#!/usr/bin/env bash
# Run the commit-msg gate over every subject already on `main`.
#
# A gate is worth what it refuses and costs what it refuses wrongly, so the
# corpus is the project's own history rather than examples written to pass.
set -euo pipefail

msg=$(mktemp)
trap 'rm -f "$msg"' EXIT

pass=0
fail=0
while IFS= read -r subject; do
  printf '%s\n' "$subject" >"$msg"
  mise x -- prek run --stage commit-msg --commit-msg-filename "$msg" >/dev/null 2>&1 && rc=0 || rc=$?
  case $rc in
    0) pass=$((pass + 1)) ;;
    1) fail=$((fail + 1)); printf 'refused %3dc  %s\n' "${#subject}" "$subject" ;;
    *) echo "the gate did not run: prek exited $rc" >&2; exit 2 ;;
  esac
done < <(git log origin/main --format='%s' --no-merges)

printf '%d accepted, %d refused\n' "$pass" "$fail"
