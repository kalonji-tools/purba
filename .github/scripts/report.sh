# shellcheck shell=bash
# How a check writes a failure a person reads.
#
#   report <summary> <detail>
#
# An annotation belongs to the check run of the job that wrote it. A job that
# a gate starts runs from the default branch, so its annotation lands there and
# never on the branch it refused. `PURBA_REPORT` is how a caller reads the same
# words back and puts them where the contributor is looking.
#
# GitHub reads only the first line of an error into the annotation. A newline
# has to become `%0A` to survive, and a literal `%` has to become `%25` before
# that, or the decoder eats it.
report() {
  [ -z "${PURBA_REPORT:-}" ] || printf '%s\n\n%s\n' "$1" "$2" >>"$PURBA_REPORT"

  if [ "${GITHUB_ACTIONS:-}" != "true" ]; then
    printf '%s\n%s\n' "$1" "$2" >&2
    return
  fi
  detail=${2//%/%25}
  printf '::error::%s%%0A%s\n' "$1" "${detail//$'\n'/%0A}"
}
