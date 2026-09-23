# shellcheck shell=bash
# How a check writes a failure a person reads.
#
#   report <summary> <detail>
#
# GitHub reads only the first line of an error into the annotation. A newline
# has to become `%0A` to survive, and a literal `%` has to become `%25` before
# that, or the decoder eats it.
report() {
  if [ "${GITHUB_ACTIONS:-}" != "true" ]; then
    printf '%s\n%s\n' "$1" "$2" >&2
    return
  fi
  detail=${2//%/%25}
  printf '::error::%s%%0A%s\n' "$1" "${detail//$'\n'/%0A}"
}
