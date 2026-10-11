#!/usr/bin/env bash
# The tracked paths that .config/gates does not mark set or unset for each gate it
# names.
#
#   the task:  mise run lint:gates
#
#   check-gates.sh
#
# Exits 1 when it refuses a path, and 2 when .config/gates cannot be read or names
# no gate.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"

top=$(git rev-parse --show-toplevel 2>/dev/null) ||
  cannot 'the marks can only be found inside a git repository.'
cd "${top}"

[[ -r .config/gates ]] ||
  cannot '.config/gates cannot be read.'

# git writes NUL-separated output, which a shell variable cannot hold, so each
# step writes a file here.
scratch=$(mktemp -d)
trap 'rm -rf "${scratch}"' EXIT

# Each word after a pattern names a gate, whichever state it gives the path.
names=$(awk '
  /^[[:space:]]*(#|$)/ { next }
  { for (i = 2; i <= NF; i++) { x = $i; sub(/^[-!]/, "", x); sub(/=.*/, "", x); print x } }
' .config/gates | sort -u)
mapfile -t gates <<<"${names}"
[[ -n "${names}" ]] ||
  cannot '.config/gates names no gate, so no path can be checked.'

# A template can carry an info/attributes file, so the empty repository takes none.
git init --quiet --template= "${scratch}/empty"
git ls-files -z >"${scratch}/files"
GIT_ATTR_NOSYSTEM=1 git -C "${scratch}/empty" -c core.attributesFile="${top}/.config/gates" \
  check-attr -z --stdin "${gates[@]}" <"${scratch}/files" >"${scratch}/marks" ||
  cannot 'git cannot resolve .config/gates.'

unmarked=()
while IFS= read -r -d '' path && IFS= read -r -d '' gate && IFS= read -r -d '' state; do
  [[ "${state}" == set || "${state}" == unset ]] || unmarked+=("${gate}: ${path}")
done <"${scratch}/marks"

[[ ${#unmarked[@]} -eq 0 ]] || refuse \
  "A tracked path is refused when .config/gates does not mark it set or unset for each gate \
the file names, because a gate that skips must know whether the path starts it. Mark the path \
\`build\` when a job of build.yml reads it, and \`-build\` when none does." \
  "${unmarked[@]}"

finish
