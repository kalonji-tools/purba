# shellcheck shell=bash
# What a record is, where its reach is, the paths a reach matches, and the line
# outside a fence that a pattern finds.
#
#   the decision:  docs/decisions/a-record-is-rewritten-not-amended.md
#                  docs/decisions/a-record-names-every-location-where-it-applies.md
#
#   is_record <path>                     prints <path> when it is a record
#   record_line <record> <ERE>           prints the number of the first line
#               [<after>]                below <after> that matches <ERE> outside
#                                        a fence. <after> defaults to 0
#   reach_lines <record>                 prints `<first> <last> reach`, the lines
#                                        the Decision Outcome opens with, or
#                                        `<first> <last> other` when they are not
#                                        a reach. Nothing with no Decision Outcome
#   reach_patterns <record>              prints each pattern of the reach, one to
#                                        a line
#   reach_is_everywhere <record>         prints `everywhere` when the reach holds
#                                        `**` or a glossary word
#   reach_hits <scratch> <paths>         prints `<record> NUL <pattern> NUL <path>
#              <record>...               NUL` for each NUL-separated path in
#                                        <paths> that a pattern matches
#   reach_match <scratch> <attributes>   prints `<attribute> NUL <path> NUL` for
#               <paths>                  each NUL-separated path a pattern matches
#
# Each line of <attributes> is `<pattern> <attribute>`. <scratch> is a directory
# the caller removes. The caller sources report.sh first.

# shellcheck source=scripts/attributes.sh
. "$(dirname "${BASH_SOURCE[0]}")/attributes.sh"

is_record() {
  [[ ! "$1" =~ ^docs/decisions/[^/.][^/]*\.md$ ]] || printf '%s\n' "$1"
}

# Sets <found> in the caller to the number of the first line of the array
# <lines> below <after> that matches <ERE> outside a fence, or to nothing.
#
# A fence of three or more backticks or tildes hides each line until a run of
# the same character, at least as long, closes it. A fence that no run closes
# runs to the end.
#
#   the fence:     https://spec.commonmark.org/0.31.2/#fenced-code-blocks
unfenced_find() {
  local -n from_lines=$1
  local pattern=$2 after=$3 line fence="" number=0 tick=$'\x60'
  printf -v "$4" '%s' ""
  for line in "${from_lines[@]}"; do
    number=$((number + 1))
    if [[ -n "${fence}" ]]; then
      [[ ! "${line}" =~ ^\ {0,3}(${fence:0:1}+)[[:space:]]*$ ||
        ${#BASH_REMATCH[1]} -lt ${#fence} ]] || fence=""
    elif [[ "${line}" =~ ^\ {0,3}(${tick}{3,}|~{3,}) ]]; then
      fence=${BASH_REMATCH[1]}
    elif [[ "${number}" -gt "${after}" && "${line}" =~ ${pattern} ]]; then
      printf -v "$4" '%s' "${number}"
      return 0
    fi
  done
}

record_line() {
  local found
  local -a content
  mapfile -t content <"$1"
  unfenced_find content "$2" "${3:-0}" found
  [[ -z "${found}" ]] || printf '%s\n' "${found}"
}

# Sets <span> to `<first> <last> reach` or `<first> <last> other`, and <reach>
# to the reach, in the caller. Both are empty when there is nothing to give. It
# runs in the caller's shell, so a record is read once for each call.
#
# The lines are those of the paragraph the Decision Outcome opens with, whole,
# because a reach can wrap. A heading where the paragraph should be gives its
# line and no reach. What is left once each location is taken out must be
# blank, because the prose rules do not read a reach.
reach_read() {
  local line at number first=0 last=0 text="" rest tick=$'\x60' one
  local -a content
  printf -v "$2" '%s' ""
  printf -v "$3" '%s' ""
  mapfile -t content <"$1"
  unfenced_find content '^## Decision Outcome$' 0 at
  [[ -n "${at}" ]] || return 0
  for ((number = at + 1; number <= ${#content[@]}; number++)); do
    line=${content[number - 1]}
    if [[ -z "${line//[[:space:]]/}" ]]; then
      [[ "${first}" -eq 0 ]] || break
      continue
    fi
    if [[ "${line}" == '#'* ]]; then
      [[ "${first}" -ne 0 ]] || first=${number} last=${number}
      break
    fi
    [[ "${first}" -ne 0 ]] || first=${number}
    last=${number}
    text+="${line}"$'\n'
  done
  [[ "${first}" -ne 0 ]] || return 0
  printf -v "$2" '%s %s other' "${first}" "${last}"
  one="${tick}[^${tick}]+${tick}|\[[^]]+\]\(\.\./\.\./CONTEXT\.md#[a-z-]+\)"
  [[ "${text}" =~ ^\*\*Reach:\*\*\ .*(${one}) ]] || return 0
  rest=${text#"**Reach:**"}
  while [[ "${rest}" =~ ${one} ]]; do
    rest=${rest/"${BASH_REMATCH[0]}"/}
  done
  [[ -z "${rest//[[:space:]]/}" ]] || return 0
  printf -v "$2" '%s %s reach' "${first}" "${last}"
  printf -v "$3" '%s' "${text%$'\n'}"
}

# Fills the array <patterns> with each code span of <reach>, in order.
reach_spans() {
  local text=$1
  local -n into_patterns=$2
  into_patterns=()
  while [[ "${text}" =~ \`([^\`]+)\` ]]; do
    into_patterns+=("${BASH_REMATCH[1]}")
    text=${text#*"${BASH_REMATCH[0]}"}
  done
}

reach_lines() {
  local span reach
  reach_read "$1" span reach
  [[ -z "${span}" ]] || printf '%s\n' "${span}"
}

reach_patterns() {
  local span reach
  local -a patterns
  reach_read "$1" span reach
  reach_spans "${reach}" patterns
  [[ ${#patterns[@]} -eq 0 ]] || printf '%s\n' "${patterns[@]}"
}

reach_is_everywhere() {
  local span reach pattern
  local -a patterns
  reach_read "$1" span reach
  reach_spans "${reach}" patterns
  if [[ "${reach}" == *'](../../CONTEXT.md#'* ]]; then
    printf 'everywhere\n'
    return 0
  fi
  for pattern in "${patterns[@]}"; do
    [[ "${pattern}" != '**' ]] || {
      printf 'everywhere\n'
      return 0
    }
  done
}

# Each pattern of each record is one attribute, so a match names both.
reach_hits() {
  local scratch=$1 paths=$2 record span reach pattern attribute name
  local -a from=() held=() patterns
  shift 2
  : >"${scratch}/reach-patterns"
  for record in "$@"; do
    reach_read "${record}" span reach
    reach_spans "${reach}" patterns
    for pattern in "${patterns[@]}"; do
      from+=("${record}")
      held+=("${pattern}")
      printf '%s r%d\n' "${pattern}" "${#held[@]}" >>"${scratch}/reach-patterns"
    done
  done
  [[ ${#held[@]} -gt 0 ]] || return 0
  reach_match "${scratch}" "${scratch}/reach-patterns" "${paths}" >"${scratch}/reach-hits"
  while IFS= read -r -d '' attribute && IFS= read -r -d '' name; do
    attribute=$((${attribute#r} - 1))
    printf '%s\0%s\0%s\0' "${from[attribute]}" "${held[attribute]}" "${name}"
  done <"${scratch}/reach-hits"
}

# A pattern is matched as git matches an attribute, which is not how it matches
# a pathspec.
reach_match() {
  local scratch=$1 attributes=$2 paths=$3 attribute name state
  attr_states "${scratch}" "${attributes}" <"${paths}" >"${scratch}/attributes"
  while IFS= read -r -d '' name && IFS= read -r -d '' attribute && IFS= read -r -d '' state; do
    [[ "${state}" != set ]] || printf '%s\0%s\0' "${attribute}" "${name}"
  done <"${scratch}/attributes"
}
