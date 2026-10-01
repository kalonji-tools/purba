#!/usr/bin/env bash
# The decision-record rules a command can decide.
#
#   the decision:  docs/decisions/a-record-is-rewritten-not-amended.md
#   the command:   mise run records
#
#   check-records.sh [directory]
#
# Exits 1 when a record breaks a rule, and 2 when this script cannot decide.
#
# It reports every rule before it exits, because a writer fixing one refusal
# should not have to run it again to find the next.
set -euo pipefail

dir=${1:-docs/decisions}

# A leading dot does not match `*`, so the template is excluded by the glob
# rather than by a name test.
shopt -s nullglob
records=("${dir}"/*.md)

[[ ${#records[@]} -gt 0 ]] || {
  printf 'no record found in %s.\n' "${dir}" >&2
  exit 2
}

required=$(printf '%s\n' \
  '## Context and Problem Statement' \
  '## Considered Options' \
  '## Decision Outcome' \
  '## Confirmation')

# Consequences is the one optional section, and it sits where the format it
# follows puts it, after the outcome and before the confirmation.
optional=$(printf '%s\n' \
  '## Context and Problem Statement' \
  '## Considered Options' \
  '## Decision Outcome' \
  '## Consequences' \
  '## Confirmation')

# The last alternative matches a table cell answering `no`, which is how a
# record states an unwired gate without writing a sentence.
admits='not wired|not written yet|does not exist yet|by hand|\| *no *\|'

broken=0

refuse() {
  printf '\n%s\n' "$1" >&2
  shift
  printf '  %s\n' "$@" >&2
  broken=1
}

found=()
for f in "${records[@]}"; do
  headings=$(grep '^## ' "${f}" || true)
  [[ "${headings}" = "${required}" ]] || [[ "${headings}" = "${optional}" ]] || found+=("${f}")
done
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A record carries four sections, in order: Context and Problem Statement, Considered Options, \
Decision Outcome, Confirmation. Consequences is optional and sits between the outcome and the \
confirmation." \
  "${found[@]}"

mapfile -t found < <(grep -HnF '<!--' "${records[@]}" || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A record reads as current state, so it carries no comment addressed to a reviewer. The \
template carries the drafting instructions, and a leading dot keeps it out of this glob." \
  "${found[@]}"

mapfile -t found < <(grep -HnE '#[0-9]+' "${records[@]}" || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A record states what is true and an issue states what happened, so record prose carries no \
issue number. Title the link with the question its ticket asks." \
  "${found[@]}"

# The two prose rules `docs/decisions/.template.md` states.
#
#   the decision:  docs/decisions/an-artifact-holds-the-minimum-that-conveys-its-point.md
#
# `**` opens bold only where it flanks, which is a left-flanking delimiter run.
# https://spec.commonmark.org/0.31.2/#left-flanking-delimiter-run
# Without that test an unbackticked `.github/**` and `/scripts/**` on one line
# pair into a bold run, and masking code spans does not reach it.
prose=$(awk '
  # A code span becomes \001. Splitting on the backtick puts every span in an
  # even field, which is what a regex cannot do: it pairs a closing backtick
  # with a later opening one.
  function mask(line,   part, n, i, out) {
    n = split(line, part, "`")
    out = part[1]
    for (i = 2; i <= n; i++) out = out (i % 2 == 0 ? "\001" : part[i])
    return out
  }

  # A sentence starts at the head of a unit, or after `.`, `!` or `?`, which a
  # closing `**` may follow. A non-alphanumeric run may precede the bold, which
  # is the allowance the Downside lead-in count already grants a list marker.
  function opens(unit, at,   head) {
    head = substr(unit, 1, at - 1)
    sub(/^.*[.!?]\**[[:space:]]+/, "", head)
    return (head ~ /^[^A-Za-z0-9\001]*$/)
  }

  # Indented, because a fence nested under a list item still opens a block, and
  # an anchor at column zero reads its contents as prose.
  /^[[:space:]]*```/ { fence = ! fence; next }
  # A heading is not a sentence. The evidence-density pass below skips one too.
  fence || /^#/ { next }
  {
    masked = mask($0)
    # A unit is a line, or a table cell once the row is split on the pipes that
    # are not inside a code span.
    units = 1
    cell[1] = masked
    if ($0 ~ /^\|/) units = split(masked, cell, /\|/)

    for (u = 1; u <= units; u++) {
      unit = cell[u]
      sub(/^[[:space:]]+/, "", unit)
      if (unit == "") continue

      # The literal character, never a \x escape: that escape is not POSIX and
      # an awk that ignores it matches nothing and reports a clean tree.
      if (index(unit, "—") && ! seen["e" FILENAME ":" FNR]++) \
        printf "emdash\t%s:%d\n", FILENAME, FNR

      rest = unit
      base = 0
      while (match(rest, /\*\*[^[:space:]][^*]*[^[:space:]]\*\*|\*\*[^[:space:]*]\*\*/)) {
        at = base + RSTART
        if (! opens(unit, at) && ! seen["b" FILENAME ":" FNR]++) \
          printf "bold\t%s:%d\n", FILENAME, FNR
        base = at + RLENGTH - 1
        rest = substr(rest, RSTART + RLENGTH)
      }
    }
  }
' "${records[@]}")

mapfile -t found < <(grep '^emdash' <<<"${prose}" | cut -f2- || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A record carries no em-dash. Use a full stop, a colon, a comma or a list. A code span and a \
fenced block are exempt, because quoting one is quoting an artifact." \
  "${found[@]}"

mapfile -t found < <(grep '^bold' <<<"${prose}" | cut -f2- || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "Bold opens a sentence and never sits inside one. A lead-in is what bold is for, and whether a \
bolded phrase is one is read by a reviewer rather than decided here." \
  "${found[@]}"

# The four Downside rules `docs/decisions/.template.md` states: the label stands
# on its own line, it never counts the costs, the costs are a list, and bold
# marks every lead-in.
#
# One pass reads each record and fills four lists, so a writer fixing one
# refusal sees the rest in the same run.
#
# A record carrying no label has no list of costs either, so the list rule is
# what an absent Downside breaks. Without that arm the other three pass a record
# that simply deletes the label.
preamble=()
counted=()
listless=()
unsignalled=()
for f in "${records[@]}"; do
  # Anchored: a record naming the label inside a table would otherwise locate
  # the label at that row, and a preamble on the real label would go unseen.
  at=$(awk '/^\*\*Downside:\*\*/{print NR; exit}' "${f}")
  if [[ -z "${at}" ]]; then
    listless+=("${f}: no Downside label")
    continue
  fi

  rest=$(sed -n "${at}s/^\*\*Downside:\*\*//p" "${f}")
  if [[ -n "${rest// /}" ]]; then
    preamble+=("${f}:${at}")
    # A count opens the preamble. Written as a word or a digit, because both
    # forms are in the corpus this rule was measured against.
    if grep -qiE '^ *(one|two|three|four|five|six|seven|eight|nine|ten|[0-9]+)\b' <<<"${rest}"; then
      counted+=("${f}:${at}")
    fi
  fi

  # To the next section heading. The Downside is the last thing in the outcome.
  costs=$(awk -v start="${at}" 'NR > start && /^## /{exit} NR > start' "${f}")
  items=$(grep -cE '^[[:space:]]*- ' <<<"${costs}" || true)
  # Occurrences rather than lines: two lead-ins on one line is the shape that
  # renders three costs where four are written.
  leadins=$({ grep -oE -- '- [^A-Za-z0-9`]*\*\*' <<<"${costs}" || true; } | wc -l)

  if [[ "${items}" -eq 0 ]]; then
    listless+=("${f}:${at}")
  elif [[ "${leadins}" -ne "${items}" ]]; then
    unsignalled+=("${f}:${at}: ${leadins} lead-ins over ${items} costs")
  fi
done

[[ ${#preamble[@]} -eq 0 ]] || refuse \
  "A Downside label stands on its own line. Its costs are the list beneath it, so a preamble is \
a sentence the reader carries while reading them." \
  "${preamble[@]}"

[[ ${#counted[@]} -eq 0 ]] || refuse \
  "A Downside label never counts its costs. A count is one more thing to keep true, and one \
record counted four costs where three rendered." \
  "${counted[@]}"

[[ ${#listless[@]} -eq 0 ]] || refuse \
  "A Downside states its costs as a list. A record with no label has no list either, so a \
missing Downside is refused here." \
  "${listless[@]}"

[[ ${#unsignalled[@]} -eq 0 ]] || refuse \
  "Bold marks every Downside lead-in, one for each cost. Fewer lead-ins than costs leaves a cost \
unmarked, and more means a cost opens mid-line and does not render." \
  "${unsignalled[@]}"

git rev-parse --git-dir >/dev/null 2>&1 || {
  printf 'a numbered-record citation can only be read inside a git repository.\n' >&2
  exit 2
}
mapfile -t found < <(git grep -InE 'ADR-[0-9]+' || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A number cannot be checked against the record it names, so source cites a record by its \
proposition." \
  "${found[@]}"

found=()
for f in "${records[@]}"; do
  confirmation=$(sed -n '/^## Confirmation/,$p' "${f}")
  printf '%s\n' "${confirmation}" | grep -qE "${admits}" || continue
  printf '%s\n' "${confirmation}" | grep -q 'purba/issues/[0-9]' || found+=("${f}")
done
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A Confirmation that says a gate is unwired names the ticket that will wire it, so the promise \
has an owner." \
  "${found[@]}"

# Evidence density, reported and never refused. `docs/decisions/.template.md`
# moves numbers out of sentences, so a record that obeys it scores bare by
# construction and a threshold would refuse the records that comply.
#
# A prose line starts at column zero and is not a heading, a table row, a list
# item, a fenced block or a comment. It is bare when it carries no digit, no
# code span and no link. An indented line is a list continuation here, because
# no record indents prose.
printf '\nevidence density: prose lines carrying no number, no code span and no link\n\n'
awk '
  /^[[:space:]]*```/ { fence = ! fence; next }
  fence || /^$/ || /^[[:space:]]/ || /^#/ || /^\|/ || /^[-*] / || /^[0-9]+\. / || /^<!--/ { next }
  { prose[FILENAME]++; lines++ }
  ! /[0-9`]/ && ! /\]\(/ { bare[FILENAME]++; barelines++ }
  END {
    for (f in prose) {
      printf "%5.1f%%  %3d of %3d  %s\n", 100 * bare[f] / prose[f], bare[f], prose[f], f
    }
    printf "all records  %.1f%%  %d of %d\n", 100 * barelines / lines, barelines, lines
  }
' "${records[@]}" | sort -rn

exit "${broken}"
