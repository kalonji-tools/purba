#!/usr/bin/env bash
# The tracked files that reach no actor, and the names in .readers that are not
# on the roster.
#
#   the decision:  docs/decisions/a-location-inherits-its-readers.md
#   the roster:    docs/decisions/an-actor-is-what-it-does-not-what-it-is.md
#   the command:   mise run lint:readers
#
#   check-readers.sh
#
# Exits 1 when it refuses a file or a name, and 2 when .readers or the roster
# cannot be read.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"

top=$(git rev-parse --show-toplevel 2>/dev/null) ||
  cannot 'the bindings can only be found inside a git repository.'
cd "${top}"

[[ -r .readers ]] ||
  cannot '.readers cannot be read.'

# git writes NUL-separated output, which a shell variable cannot hold, so each
# step writes a file here.
scratch=$(mktemp -d)
trap 'rm -rf "${scratch}"' EXIT

# Only the table under `| actor | mindset |` is the roster.
record=docs/decisions/an-actor-is-what-it-does-not-what-it-is.md
slugs=$(awk '
  /^\| actor \| mindset \|$/ { table = 1; next }
  table && !/^\|/ { exit }
  table && match($0, /CONTEXT\.md#[a-z-]+\)/) { print substr($0, RSTART + 11, RLENGTH - 12) }
' "${record}" 2>/dev/null) || slugs=""
declare -A actor=()
while IFS= read -r slug; do
  [[ -z "${slug}" ]] || actor[${slug}]=1
done <<<"${slugs}"
[[ ${#actor[@]} -gt 0 ]] ||
  cannot "the roster cannot be read from ${record}."

# Every name .readers writes, and each macro it defines, read off the lines.
# This matches no pattern, so it catches a name on a pattern that reaches no
# file.
problems=$(slugs="${slugs}" awk '
  BEGIN { n = split(ENVIRON["slugs"], list, "\n"); for (i = 1; i <= n; i++) actor[list[i]] = 1 }
  /^[[:space:]]*(#|$)/ { next }
  $1 ~ /^\[attr\]/ { macro[substr($1, 7)] = 1 }
  { for (i = 2; i <= NF; i++) { x = $i; sub(/^[-!]/, "", x); sub(/=.*/, "", x); used[x] = 1 } }
  END {
    for (x in macro) if (x in actor) print "shadow", x
    for (x in used) if (!(x in actor) && !(x in macro)) print "off", x
  }
' .readers | sort)
off_roster=$(sed -n 's/^off //p' <<<"${problems}")
shadowed=$(sed -n 's/^shadow //p' <<<"${problems}")

# A template can carry an info/attributes file, so the empty repository takes none.
git init --quiet --template= "${scratch}/empty"
git ls-files -z >"${scratch}/files"
GIT_ATTR_NOSYSTEM=1 git -C "${scratch}/empty" -c core.attributesFile="${top}/.readers" \
  check-attr -z --all --stdin <"${scratch}/files" >"${scratch}/attributes" ||
  cannot 'git cannot resolve .readers.'

declare -A reached=()
while IFS= read -r -d '' path && IFS= read -r -d '' name && IFS= read -r -d '' state; do
  [[ "${state}" != set || -z "${actor[${name}]:-}" ]] || reached[${path}]=1
done <"${scratch}/attributes"

unbound=()
while IFS= read -r -d '' path; do
  [[ -n "${reached[${path}]:-}" ]] || unbound+=("${path}")
done <"${scratch}/files"

[[ ${#unbound[@]} -eq 0 ]] || refuse \
  "A tracked file is refused when it reaches no actor, because every artifact has a named \
reader. Decide whether it should exist, what it serves and for whom, and bind that actor in \
.readers." \
  "${unbound[@]}"

[[ -z "${off_roster}" ]] || refuse \
  "A name in .readers is refused when it is neither an actor nor a macro, because a reader \
is an actor and the roster names every actor." \
  "${off_roster}"

[[ -z "${shadowed}" ]] || refuse \
  "A macro is refused when its name is an actor's, because .readers could not \
then tell the two apart." \
  "${shadowed}"

finish
