#!/usr/bin/env bash
# The lints of each lane that a pull request started, as one `mise run`.
#
#   the decision:  docs/decisions/ci-runs-a-lane-of-the-lint-only-when-a-path-it-reads-changes.md
#   the caller:    .github/workflows/ci.yml
#   the lanes:     .config/lanes
#
#   lanes.sh --names              prints each lane but `always`, once
#   lanes.sh --tasks [<lane>...]  prints the lints of the lanes named and of
#                                 `always`, joined by ` ::: `
#
# A lint is a task of .config/tasks.toml whose name starts with `lint:`, as for
# `mise run lint`. A lint that .config/lanes does not name runs on every pull request.
# Standard error names the lanes that started and each lint that skipped.
#
# Exits 0 when it decided, and 2 when it cannot run.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/../../scripts/report.sh"

top=$(git rev-parse --show-toplevel 2>/dev/null) ||
  cannot 'the lanes can only be found inside a git repository.'
cd "${top}"

lanes=.config/lanes
[[ -r "${lanes}" ]] || cannot "${lanes} cannot be read."

case "${1:-}" in
  --names)
    awk '!/^[[:space:]]*(#|$)/ { for (i = 2; i <= NF; i++) if ($i != "always") print $i }' \
      "${lanes}" | sort -u
    ;;
  --tasks)
    shift
    [[ -r .config/tasks.toml ]] || cannot '.config/tasks.toml cannot be read.'
    all=$(sed -n -E 's/^\["(lint:[a-z-]+)"\]$/\1/p' .config/tasks.toml)
    [[ -n "${all}" ]] || cannot '.config/tasks.toml defines no lint.'
    started=" always $* "
    picked=() skipped=()
    for task in ${all}; do
      named=$(awk -v t="${task}" '$1 == t { for (i = 2; i <= NF; i++) print $i }' "${lanes}")
      hit=0
      [[ -n "${named}" ]] || hit=1
      for lane in ${named}; do
        [[ "${started}" != *" ${lane} "* ]] || hit=1
      done
      if ((hit)); then picked+=("${task}"); else skipped+=("${task}"); fi
    done
    echo "lanes that started: always${*:+ $*}" >&2
    echo "lints skipped: ${skipped[*]:-none}" >&2
    out=""
    for task in "${picked[@]}"; do out+="${out:+ ::: }${task}"; done
    echo "${out}"
    ;;
  *)
    cannot 'usage: lanes.sh --names | --tasks [<lane>...]'
    ;;
esac
