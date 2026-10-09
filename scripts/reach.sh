# shellcheck shell=bash
# How a script reads the reach of a record, and the paths a reach matches.
#
#   the decision:  docs/decisions/a-record-names-every-location-where-it-applies.md
#
#   reach_paragraph                        reads a record on standard input, and
#                                          prints the line its reach starts on,
#                                          then the reach
#   reach_match <scratch> <attributes>     prints `<attribute> NUL <path> NUL` for
#               <paths>                    each NUL-separated path a pattern matches
#
# Each line of <attributes> is `<pattern> <attribute>`. <scratch> is a directory
# the caller removes. The caller sources report.sh first.

# The first paragraph under the Decision Outcome heading, whole, because a reach
# can wrap. A heading where the paragraph should be gives its line and nothing
# else, and a record with no such heading gives nothing.
reach_paragraph() {
  awk '
    /^## Decision Outcome$/ { at = 1; next }
    at && !NF { if (shown) exit; next }
    at && /^#/ { if (!shown) print NR; exit }
    at { if (!shown) print NR; shown = 1; print }
  '
}

# A pattern is matched as git matches an attribute, which is not how it matches
# a pathspec. A template can carry an info/attributes file, so the empty
# repository takes none.
reach_match() {
  local scratch=$1 attributes=$2 paths=$3 attribute name state
  git init --quiet --template= "${scratch}/empty" ||
    cannot "git cannot make the empty repository a reach is matched in."
  GIT_ATTR_NOSYSTEM=1 git -C "${scratch}/empty" -c core.attributesFile="${attributes}" \
    check-attr -z --all --stdin <"${paths}" >"${scratch}/attributes" ||
    cannot 'git cannot match the patterns of a reach.'
  while IFS= read -r -d '' name && IFS= read -r -d '' attribute && IFS= read -r -d '' state; do
    [[ "${state}" != set ]] || printf '%s\0%s\0' "${attribute}" "${name}"
  done <"${scratch}/attributes"
}
