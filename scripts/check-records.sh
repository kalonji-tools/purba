#!/usr/bin/env bash
# The decision-record rules a command can decide.
#
#   the decision:  docs/decisions/a-record-is-rewritten-not-amended.md
#   the command:   mise run records
#
#   check-records.sh [directory]
#
# Exits 1 when a record breaks a rule, and 2 when this script cannot decide.
#
# It reports every rule before it exits, because a writer fixing one refusal
# should not have to run it again to find the next.
set -euo pipefail

dir=${1:-docs/decisions}


# A leading dot does not match `*`, so the template is excluded by the glob
# rather than by a name test.
shopt -s nullglob
records=("$dir"/*.md)

[ ${#records[@]} -gt 0 ] || {
  printf 'no record found in %s.\n' "$dir" >&2
  exit 2
}

required=$'## Context and Problem Statement\n## Considered Options\n## Decision Outcome\n## Confirmation'

# Consequences is the one optional section, and it sits where the format it
# follows puts it, after the outcome and before the confirmation.
optional=$'## Context and Problem Statement\n## Considered Options\n## Decision Outcome\n## Consequences\n## Confirmation'

# The last alternative matches a table cell answering `no`, which is how a
# record states an unwired gate without writing a sentence.
admits='not wired|not written yet|does not exist yet|by hand|\| *no *\|'

broken=0

refuse() {
  printf '\n%s\n' "$1" >&2
  shift
  printf '  %s\n' "$@" >&2
  broken=1
}

found=()
for f in "${records[@]}"; do
  headings=$(grep '^## ' "$f" || true)
  [ "$headings" = "$required" ] || [ "$headings" = "$optional" ] || found+=("$f")
done
[ ${#found[@]} -eq 0 ] || refuse \
  'A record carries four sections, in order: Context and Problem Statement, Considered Options, Decision Outcome, Confirmation. Consequences is optional and sits between the outcome and the confirmation.' \
  "${found[@]}"

mapfile -t found < <(grep -HnE '#[0-9]+' "${records[@]}" || true)
[ ${#found[@]} -eq 0 ] || refuse \
  'A record states what is true and an issue states what happened, so record prose carries no issue number. Title the link with the question its ticket asks.' \
  "${found[@]}"

git rev-parse --git-dir >/dev/null 2>&1 || {
  printf 'a numbered-record citation can only be read inside a git repository.\n' >&2
  exit 2
}
mapfile -t found < <(git grep -InE 'ADR-[0-9]+' || true)
[ ${#found[@]} -eq 0 ] || refuse \
  'A number cannot be checked against the record it names, so source cites a record by its proposition.' \
  "${found[@]}"

found=()
for f in "${records[@]}"; do
  confirmation=$(sed -n '/^## Confirmation/,$p' "$f")
  printf '%s\n' "$confirmation" | grep -qE "$admits" || continue
  printf '%s\n' "$confirmation" | grep -q 'purba/issues/[0-9]' || found+=("$f")
done
[ ${#found[@]} -eq 0 ] || refuse \
  'A Confirmation that says a gate is unwired names the ticket that will wire it, so the promise has an owner.' \
  "${found[@]}"

exit "$broken"
