# shellcheck shell=bash
# How a check writes a failure a person reads.
#
#   the decision:  docs/decisions/a-refusal-states-the-rule-why-it-holds-and-where-it-was-found.md
#
#   report <summary> <detail>
#
#   PURBA_REPORT   a file the plain text is appended to as well, when set
#
# A refusal states the rule that must hold, and why it holds, in its summary. It
# says what to do where the check knows the repair. Its detail says where the
# check found the breach. It names no record, because a gate reads a cited path
# only in a comment.
#
# GitHub reads only the first line of an error into the annotation. A newline
# has to become `%0A` to survive, and a literal `%` has to become `%25` before
# that, or the decoder eats it.
report() {
  [[ -z "${PURBA_REPORT:-}" ]] || printf '%s\n\n%s\n' "$1" "$2" >>"${PURBA_REPORT}"

  if [[ "${GITHUB_ACTIONS:-}" != "true" ]]; then
    printf '%s\n%s\n' "$1" "$2" >&2
    return
  fi
  detail=${2//%/%25}
  printf '::error::%s%%0A%s\n' "$1" "${detail//$'\n'/%0A}"
}
