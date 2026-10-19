# shellcheck shell=bash
# How a script matches a gitattributes pattern as git matches it, and the names
# an attributes file writes.
#
#   the decision:  docs/decisions/a-record-names-every-location-where-it-applies.md
#                  docs/decisions/a-location-inherits-its-readers.md
#
#   attr_states <scratch> <attributes>   prints `<path> NUL <attr> NUL <state> NUL`
#               [<attr>...]              for each NUL-separated path on standard
#                                        input, and each <attr>. With no <attr>,
#                                        each attribute the file gives the path
#   attr_names <attributes>              prints each name after a pattern or a
#                                        macro, once
#
# <scratch> is a directory the caller removes. A relative <attributes> is read
# from the caller's directory, and a repository the caller's environment names
# does not change a match. The caller sources report.sh first, and a git that
# cannot match exits 2.

# A template can carry an info/attributes file, so the empty repository takes
# none.
attr_states() (
  local scratch=$1 attributes=$2 asked=(--all) names
  local -a handed
  [[ $# -le 2 ]] || asked=("${@:3}")
  [[ "${attributes}" == /* ]] || attributes="${PWD}/${attributes}"
  names=$(git rev-parse --local-env-vars) ||
    cannot 'git cannot name the variables that point it at a repository.'
  mapfile -t handed <<<"${names}"
  unset "${handed[@]}"
  git init --quiet --template= "${scratch}/empty" ||
    cannot 'git cannot make the empty repository a pattern is matched in.'
  GIT_ATTR_NOSYSTEM=1 git -C "${scratch}/empty" -c core.attributesFile="${attributes}" \
    check-attr -z --stdin "${asked[@]}" ||
    cannot "git cannot match the patterns in ${attributes}."
)

attr_names() {
  local written
  written=$(awk '
    /^[[:space:]]*(#|$)/ { next }
    { for (i = 2; i <= NF; i++) { x = $i; sub(/^[-!]/, "", x); sub(/=.*/, "", x); print x } }
  ' "$1")
  [[ -z "${written}" ]] || sort -u <<<"${written}"
}
