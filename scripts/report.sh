# shellcheck shell=bash
# How a check writes a failure a person reads.
#
#   the decision:  docs/decisions/a-refusal-states-the-rule-why-it-holds-and-where-it-was-found.md
#
#   refuse <summary> <finding>...   writes a refusal, each line of a finding indented 2 spaces
#   warn <summary> <finding>...     writes a warning laid out as a refusal, and refuses nothing
#   finish                          exits 1 after a refusal, and 0 with none
#   cannot <message> [detail]       writes why the check cannot run, and exits 2
#   report <summary> <detail>       writes the refusal that refuse lays out
#
#   PURBA_REPORT   a file the plain text is appended to as well, when set. A
#                  warning is not appended, because a warning stops no job.
#
# A refusal states the rule that must hold, and why it holds, in its summary. It
# says what to do where the check knows the repair. Its detail says where the
# check found the breach. It names no record, because a gate reads a cited path
# only in a comment. A message for a check that cannot run is not a refusal, so
# that test is not applied to it.
#
# GitHub reads only the first line of an error into the annotation. A newline
# has to become `%0A` to survive, and a literal `%` has to become `%25` before
# that, or the decoder eats it.

refused=0
apart=0

refuse() {
  lay_out "${@:2}"
  report "$1" "${laid}"
  refused=1
}

warn() {
  lay_out "${@:2}"
  emit "$1" "${laid}" warning
}

# Sets `laid` to each line of each finding, indented 2 spaces. The blank line
# keeps one message apart from the one before it in a terminal.
lay_out() {
  local finding line
  [[ "${apart}" -eq 0 ]] || printf '\n' >&2
  apart=1
  laid=""
  for finding in "$@"; do
    while IFS= read -r line; do
      laid+="${line:+  ${line}}"$'\n'
    done <<<"${finding}"
  done
  laid=${laid%$'\n'}
}

finish() {
  exit "${refused}"
}

cannot() {
  emit "$1" "${2:-}"
  exit 2
}

report() {
  if [[ "$1" =~ docs/decisions/\.?[[:alnum:]] ]]; then
    printf 'a refusal names no record, and this summary names one:\n%s\n' "$1" >&2
    exit 2
  fi
  emit "$1" "$2"
}

# A message with no detail gets no detail line on a terminal or in an annotation.
emit() {
  local summary=$1 detail=$2 level=${3:-error}
  [[ -z "${PURBA_REPORT:-}" || "${level}" != error ]] ||
    printf '%s\n\n%s\n' "${summary}" "${detail}" >>"${PURBA_REPORT}"

  if [[ "${GITHUB_ACTIONS:-}" != "true" ]]; then
    printf '%s\n' "${summary}" >&2
    [[ -z "${detail}" ]] || printf '%s\n' "${detail}" >&2
    return
  fi
  [[ -z "${detail}" ]] || detail=%0A${detail//%/%25}
  printf '::%s::%s%s\n' "${level}" "${summary}" "${detail//$'\n'/%0A}" >&2
}
