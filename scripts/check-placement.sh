#!/usr/bin/env bash
# The directory a script belongs in, decided from the callers it has.
#
#   the decision:  docs/decisions/only-github-runs-what-lives-under-github.md
#   the command:   mise run lint:placement
#
#   check-placement.sh
#
# Exits 1 when a script sits in the wrong directory, and 2 when it cannot decide.
#
# It reports every script before it exits, because an author repairing one
# placement should not have to run it again to find the next.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"

git rev-parse --git-dir >/dev/null 2>&1 ||
  cannot 'a caller can only be read inside a git repository.'

# Only what can run a script counts, so `docs/` is deliberately absent.
#
# Each list is read into a variable of its own rather than through a process
# substitution, because that discards the exit status, and a masked git failure
# has already made `check-replayable.sh` misreport once.
# https://www.shellcheck.net/wiki/SC2312
scripts_list=$(git ls-files '*.sh' 'scripts/*.rs' '.github/*.rs') ||
  cannot 'the tracked scripts cannot be listed.'
callers_list=$(git ls-files '.github/workflows/*.yml' '*.sh' 'tasks.toml' 'prek.toml') ||
  cannot 'the tracked callers cannot be listed.'

[[ -n "${scripts_list}" ]] ||
  cannot 'no script is tracked, so no placement can be decided.'

# The caller glob covers `*.sh`, so this list is never empty while the one
# above is not.
mapfile -t scripts <<<"${scripts_list}"
mapfile -t callers <<<"${callers_list}"

stranded=()
unreachable=()

for script in "${scripts[@]}"; do
  mine=person
  [[ "${script}" != .github/* ]] || mine=github

  same=0
  across=()
  for caller in "${callers[@]}"; do
    [[ "${caller}" != "${script}" ]] || continue

    # A sibling is reached as `"$(dirname "$0")/report.sh"`, which carries no
    # directory, so the basename is what a caller is keyed on -- and the slash
    # is what stops a bare mention of the name from counting as one.
    grep -vE '^[[:space:]]*#' "${caller}" | grep -qF "/${script##*/}" || continue

    theirs=person
    [[ "${caller}" != .github/* ]] || theirs=github

    if [[ "${theirs}" = "${mine}" ]]; then
      same=$((same + 1))
    else
      across+=("${caller}")
    fi
  done

  [[ ${same} -eq 0 ]] || continue

  if [[ ${#across[@]} -gt 0 ]]; then
    stranded+=("${script} is named only by ${across[*]}")
  elif [[ "${mine}" = github ]]; then
    unreachable+=("${script}")
  fi
done

[[ ${#stranded[@]} -eq 0 ]] || refuse \
  "A script is refused when every caller of it lives in the other half. Only GitHub runs \
what lives under .github, and a script a person or this project's own tooling runs lives in \
scripts/." \
  "${stranded[@]}"

[[ ${#unreachable[@]} -eq 0 ]] || refuse \
  "A script under .github that nothing under .github names is reached by nothing, because \
the runners there are enumerable. A script a person runs lives in scripts/, where the person is a \
caller this command cannot see." \
  "${unreachable[@]}"

finish
