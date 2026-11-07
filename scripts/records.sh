# shellcheck shell=bash
# What a record is, where its reach is, the paths a reach matches, the areas a
# record names, the marks its options carry, and the comment of the template.
#
#   the decision:  docs/decisions/a-record-is-rewritten-not-amended.md
#                  docs/decisions/a-record-names-every-location-where-it-applies.md
#   the areas:
# docs/decisions/a-record-names-its-area-and-docs-readme-lists-the-records-by-area.md
#   the marks:
# docs/decisions/the-decision-sentence-is-the-winner-and-considered-options-holds-only-what-lost.md
#   the template:  docs/decisions/a-register-belongs-to-one-location.md
#
#   is_record <path>                     prints <path> when it is a record
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
#   area_names <landscape>               prints each area, one to a line, in the
#                                        order of the index, and nothing when the
#                                        landscape names no part or concern
#   area_read <record> <line> <areas>    sets <line> to the number of the line
#                                        after the reach, and <areas> to each
#                                        area that line names, one to a line
#   decision_read <record> <line> <text> sets <line> to the number of the first
#                                        line after the area, and <text> to that
#                                        line, or both to nothing with no area
#   option_marks <record>                prints `<line> <word>` for each option of
#                                        Considered Options whose text after its
#                                        lead-in opens with <word>, Chosen or
#                                        Rejected
#   template_comment <template>          prints <template> with each line outside
#                                        its opening comment left blank. It fails
#                                        unless its first line is `<!--` alone,
#                                        and a later line is `-->` alone
#
# Each line of <attributes> is `<pattern> <attribute>`. <scratch> is a directory
# the caller removes. The caller sources report.sh first.

# shellcheck source=scripts/attributes.sh
. "$(dirname "${BASH_SOURCE[0]}")/attributes.sh"

is_record() {
  [[ ! "$1" =~ ^docs/decisions/[^/.][^/]*\.md$ ]] || printf '%s\n' "$1"
}

# Moves the fence <fence> of the caller past <line>, and succeeds when the line
# opens a fence, sits inside one or closes it.
#
# A fence of three or more backticks or tildes hides each line until a run of
# the same character, at least as long, closes it. A fence that no run closes
# runs to the end.
#
#   the fence:     https://spec.commonmark.org/0.31.2/#fenced-code-blocks
fence_step() {
  local -n open_fence=$2
  local tick=$'\x60'
  if [[ -n "${open_fence}" ]]; then
    [[ ! "$1" =~ ^\ {0,3}(${open_fence:0:1}+)[[:space:]]*$ ||
      ${#BASH_REMATCH[1]} -lt ${#open_fence} ]] || open_fence=""
    return 0
  fi
  [[ "$1" =~ ^\ {0,3}(${tick}{3,}|~{3,}) ]] || return 1
  open_fence=${BASH_REMATCH[1]}
}

# Sets <found> in the caller to the number of the first line of the array
# <lines> that matches <ERE> outside a fence, or to nothing.
unfenced_find() {
  local -n from_lines=$1
  local pattern=$2 line fence="" number=0
  printf -v "$3" '%s' ""
  for line in "${from_lines[@]}"; do
    number=$((number + 1))
    if fence_step "${line}" fence; then
      continue
    elif [[ "${line}" =~ ${pattern} ]]; then
      printf -v "$3" '%s' "${number}"
      return 0
    fi
  done
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
  unfenced_find content '^## Decision Outcome$' at
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

# A part or a concern is the bold name that opens a bullet of its section. The
# four areas outside the landscape sit around them, in the order of the index.
area_names() {
  awk '
    /^## / {
      section = ($0 == "## The parts and their dependencies" ||
        $0 == "## The cross-cutting concerns")
    }
    section && match($0, /^- \*\*[^*]+\.\*\*/) { names[++count] = substr($0, 5, RLENGTH - 7) }
    END {
      if (count == 0) exit
      print "design"
      for (i = 1; i <= count; i++) print names[i]
      print "workflow"
      print "writing"
      print "tooling"
    }
  ' "$1"
}

# Both are empty when the record has no reach, because the line after a broken
# reach is not yet the line after a reach. <areas> is empty when the line is no
# area line, or names no area.
area_read() {
  local span reach last number line word list=""
  local -a content words
  printf -v "$2" '%s' ""
  printf -v "$3" '%s' ""
  reach_read "$1" span reach
  [[ "${span}" == *' reach' ]] || return 0
  read -r _ last _ <<<"${span}"
  mapfile -t content <"$1"
  for ((number = last + 1; number <= ${#content[@]}; number++)); do
    [[ -z "${content[number - 1]//[[:space:]]/}" ]] || break
  done
  printf -v "$2" '%s' "${number}"
  line=${content[number - 1]:-}
  [[ "${line}" =~ ^\*\*Area:\*\*(.*)$ ]] || return 0
  IFS=, read -ra words <<<"${BASH_REMATCH[1]}"
  for word in "${words[@]}"; do
    word=${word#"${word%%[![:space:]]*}"}
    word=${word%"${word##*[![:space:]]}"}
    [[ -z "${word}" ]] || list+="${word}"$'\n'
  done
  printf -v "$3" '%s' "${list%$'\n'}"
}

# An option inside a fence is quoted, so it carries no mark. A lead-in ends at
# the first `**` after the one that opens it.
option_marks() {
  # shellcheck disable=SC2034 # fence_step writes it through a name reference
  local line fence="" number=0 inside=0 rest
  while IFS= read -r line || [[ -n "${line}" ]]; do
    number=$((number + 1))
    if fence_step "${line}" fence; then
      continue
    elif [[ "${line}" == '## '* ]]; then
      inside=0
      [[ "${line}" != '## Considered Options' ]] || inside=1
    elif ((inside)) && [[ "${line}" =~ ^[[:space:]]*[-*+][[:space:]]+\*\* ]]; then
      rest=${line#*'**'}
      [[ "${rest}" == *'**'* ]] || continue
      rest=${rest#*'**'}
      rest=${rest#"${rest%%[![:space:]]*}"}
      if [[ "${rest}" =~ ^(Chosen|Rejected)([^[:alnum:]]|$) ]]; then
        printf '%s %s\n' "${number}" "${BASH_REMATCH[1]}"
      fi
    fi
  done <"$1"
}

# The names of the locals differ from any name a caller passes, because
# `printf -v` writes the nearest variable of that name.
decision_read() {
  local area_at area_list number
  local -a content
  printf -v "$2" '%s' ""
  printf -v "$3" '%s' ""
  area_read "$1" area_at area_list
  [[ -n "${area_list}" ]] || return 0
  mapfile -t content <"$1"
  for ((number = area_at + 1; number <= ${#content[@]}; number++)); do
    [[ -z "${content[number - 1]//[[:space:]]/}" ]] || break
  done
  printf -v "$2" '%s' "${number}"
  printf -v "$3" '%s' "${content[number - 1]:-}"
}

# A comment opens on a `<!--` line and closes on a `-->` line. The two marker
# lines and each line outside the comment print blank, so a line that a refusal
# names is the template's own. Nothing prints until the comment closes.
template_comment() {
  awk '
    NR == 1 {
      if ($0 !~ /^<!--[[:space:]]*$/) exit
      inside = 1
      out[NR] = ""
      next
    }
    inside && /^-->[[:space:]]*$/ { inside = 0; closed = 1; out[NR] = ""; next }
    { out[NR] = inside ? $0 : "" }
    END {
      if (!closed) exit 1
      for (i = 1; i <= NR; i++) print out[i]
    }
  ' "$1"
}
