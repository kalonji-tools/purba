#!/usr/bin/env bash
# The decision-record rules a command can decide.
#
#   the decision:  docs/decisions/a-record-is-rewritten-not-amended.md
#   the task:      mise run lint:records
#
#   check-records.sh [directory]
#
# Exits 1 when a record breaks a rule, and 2 when this script cannot decide.
#
# It reports every rule before it exits, because a writer fixing one refusal
# should not have to run it again to find the next.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"
# shellcheck source=scripts/reach.sh
. "$(dirname "$0")/reach.sh"

dir=${1:-docs/decisions}

# A leading dot does not match `*`, so the template is excluded by the glob
# rather than by a name test.
shopt -s nullglob
records=("${dir}"/*.md)

[[ ${#records[@]} -gt 0 ]] || cannot "no record found in ${dir}."

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

found=()
for f in "${records[@]}"; do
  headings=$(grep '^## ' "${f}" || true)
  [[ "${headings}" = "${required}" ]] || [[ "${headings}" = "${optional}" ]] || found+=("${f}")
done
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A record carries four sections, in order: Context and Problem Statement, Considered Options, \
Decision Outcome, Confirmation. They are a public Markdown convention cut to its minimum, and \
Confirmation keeps a record from going stale. Consequences is optional and sits between the \
outcome and the confirmation." \
  "${found[@]}"

# A record with no Decision Outcome heading is refused above.
#
#   the decision:  docs/decisions/a-record-names-every-location-where-it-applies.md
tick=$'\x60'
one="${tick}[^${tick}]+${tick}|\[[^]]+\]\(\.\./\.\./CONTEXT\.md#[a-z-]+\)"
location="^\*\*Reach:\*\* .*(${one})"
declare -A reach=()
found=()
for f in "${records[@]}"; do
  paragraph=$(reach_paragraph <"${f}")
  [[ -n "${paragraph}" ]] || continue
  text=""
  [[ "${paragraph}" != *$'\n'* ]] || text=${paragraph#*$'\n'}
  # What is left once each location is taken out must be blank, because the
  # prose rules do not read a reach.
  rest=${text#"**Reach:**"}
  while [[ "${rest}" =~ ${one} ]]; do
    rest=${rest/"${BASH_REMATCH[0]}"/}
  done
  if [[ "${text}" =~ ${location} && -z "${rest//[[:space:]]/}" ]]; then
    reach[${f}]=${text}
  else
    found+=("${f}:${paragraph%%$'\n'*}")
  fi
done
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A Decision Outcome opens with a reach: a **Reach:** paragraph that names every location where \
a change makes the record apply, and holds nothing else. A record with no reach names no \
location, so no change can be matched to it, and the prose rules do not read a reach. Write a \
path as a gitattributes pattern in a code span, and a location with no path as its glossary \
word, linked to its CONTEXT.md entry." \
  "${found[@]}"

mapfile -t found < <(grep -HnF '<!--' "${records[@]}" || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A record reads as current state, so it carries no comment addressed to a reviewer. The \
template carries the drafting instructions, and a leading dot keeps it out of this glob." \
  "${found[@]}"

mapfile -t found < <(grep -HnE '#[0-9]+' "${records[@]}" || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A record states what is true and an issue states what happened, so record prose carries no \
issue number. Title the link with the question its issue asks." \
  "${found[@]}"

# The prose rules, read through CommonMark by a cargo script. It prints one
# tagged line for each finding, and each tag below gives its refusal.
#
#   the decision:  docs/decisions/an-artifact-holds-the-minimum-that-conveys-its-point.md
here=$(cd "$(dirname "$0")" && pwd)
prose=$(cargo -Zscript --config "resolver.lockfile-path=\"${here}/check-prose/Cargo.lock\"" \
  run --quiet --release --locked --manifest-path "${here}/check-prose/check-prose.rs" \
  --target-dir "${here}/../target/scripts" -- \
  "${here}/check-prose/participles.txt" "${records[@]}") ||
  cannot 'the prose of a record cannot be read.'

mapfile -t found < <(grep '^emdash' <<<"${prose}" | cut -f2- || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A record carries no em-dash, because an em-dash joins two ideas in one sentence. Use a full \
stop, a colon, a comma or a list. A code span and a fenced block are exempt, because quoting one \
is quoting an artifact." \
  "${found[@]}"

mapfile -t found < <(grep '^bold' <<<"${prose}" | cut -f2- || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "Bold opens a sentence and never sits inside one. A lead-in is what bold is for, and whether a \
bolded phrase is one is read by a reviewer rather than decided here." \
  "${found[@]}"

mapfile -t found < <(grep '^wrap' <<<"${prose}" | cut -f2- || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A sentence stays on one line. A record is rewritten in place, and a wrapped paragraph reflows \
on a one word edit and buries the change in the diff. The line named is where the sentence \
breaks." \
  "${found[@]}"

mapfile -t found < <(grep '^long' <<<"${prose}" | cut -f2- || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A sentence in descriptive text runs to 25 words, a limit borrowed from Simplified Technical \
English. A link counts as one word and so does a code span, because a reader reads each as one \
thing." \
  "${found[@]}"

mapfile -t found < <(grep '^para' <<<"${prose}" | cut -f2- || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A paragraph runs to six sentences, a limit borrowed from Simplified Technical English. The \
line named is where the seventh lands, and a blank line splits the paragraph." \
  "${found[@]}"

mapfile -t found < <(grep '^ing' <<<"${prose}" | cut -f2- || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "An -ing form is a technical noun here and never a verb, a rule borrowed from Simplified \
Technical English. A gerund after a form of be, a preposition or a conjunction becomes a \
finite clause." \
  "${found[@]}"

# Harper's tagger decides the tense rule.
#
#   the decision:  docs/decisions/a-part-of-speech-tagger-decides-the-tense-rule.md
mapfile -t found < <(grep '^tense' <<<"${prose}" | cut -f2- || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A record uses the simple tenses, a rule borrowed from Simplified Technical English. A modal \
with the bare verb is one of them, so must be run stands and has run does not." \
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
  "A record states its costs as a list under a Downside label, because a decision without its \
cost is advocacy, not a record. A record with no label has no list either, so a missing Downside \
is refused here." \
  "${listless[@]}"

[[ ${#unsignalled[@]} -eq 0 ]] || refuse \
  "Bold marks every Downside lead-in, one for each cost. Fewer lead-ins than costs leaves a cost \
unmarked, and more means a cost opens mid-line and does not render." \
  "${unsignalled[@]}"

mapfile -t found < <(grep -HnE 'ADR-[0-9]+' "${records[@]}" || true)
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A number cannot be checked against the record it names, so a record cites another by its \
proposition." \
  "${found[@]}"

found=()
for f in "${records[@]}"; do
  confirmation=$(sed -n '/^## Confirmation/,$p' "${f}")
  printf '%s\n' "${confirmation}" | grep -qE "${admits}" || continue
  printf '%s\n' "${confirmation}" | grep -q 'purba/issues/[0-9]' || found+=("${f}")
done
if [[ ${#found[@]} -gt 0 ]]; then
  printf '\na Confirmation that admits an unwired gate and names no issue\n\n'
  printf '  %s\n' "${found[@]}"
fi

# Each pattern a reach writes in a code span is matched against the tracked
# files. Outside a git repository no file is tracked, and nothing is
# matched.
if top=$(git rev-parse --show-toplevel 2>/dev/null); then
  # git writes NUL-separated output, which a shell variable cannot hold.
  scratch=$(mktemp -d)
  trap 'rm -rf "${scratch}"' EXIT

  # One attribute for each pattern, so each one is matched on its own.
  patterns=()
  for f in "${records[@]}"; do
    line=${reach[${f}]:-}
    while [[ "${line}" =~ \`([^\`]+)\` ]]; do
      patterns+=("${f}: ${BASH_REMATCH[1]}")
      printf '%s p%d\n' "${BASH_REMATCH[1]}" "${#patterns[@]}" >>"${scratch}/reaches"
      line=${line#*"${BASH_REMATCH[0]}"}
    done
  done

  if [[ ${#patterns[@]} -gt 0 ]]; then
    git -C "${top}" ls-files -z >"${scratch}/files" ||
      cannot "git cannot list the tracked files."
    reach_match "${scratch}" "${scratch}/reaches" "${scratch}/files" >"${scratch}/matched"

    declare -A matched=()
    while IFS= read -r -d '' attribute && IFS= read -r -d '' _; do
      matched[${attribute}]=1
    done <"${scratch}/matched"

    found=()
    for i in "${!patterns[@]}"; do
      [[ -n "${matched[p$((i + 1))]:-}" ]] || found+=("${patterns[i]}")
    done
    if [[ ${#found[@]} -gt 0 ]]; then
      printf '\na pattern in a reach that matches no tracked file\n\n'
      printf '  %s\n' "${found[@]}"
    fi
  fi
fi

# Three borrowed rules a command cannot decide, reported and never
# refused. The active voice rule admits the passive where the agent is
# unknown, which is a judgement, and an article left out is not decidable at
# all: the count below says how many are present, never how many are missing.
tally=$(grep -m1 '^tally' <<<"${prose}") || tally=''
read -r _ sentences passives arts formals wordcount formallist <<<"${tally}"

# A record carrying no prose at all is already refused above, for having no
# Downside list. Dividing by its zero sentences writes an error into this
# report, between a refusal and the numbers that follow it.
pct=0
[[ "${sentences:-0}" -eq 0 ]] || pct=$((100 * passives / sentences))

printf '\nthe borrowed rules a command reports and never refuses\n\n'
printf '  %-28s %5d of %5d sentences  %d%%\n' \
  'in the passive voice' "${passives:-0}" "${sentences:-0}" "${pct}"
printf '  %-28s %5d in %5d words\n' 'articles' "${arts}" "${wordcount}"
printf '  %-28s %5d in %5d words\n' 'formal words' "${formals}" "${wordcount}"
[[ -z "${formallist}" ]] || printf '  %-28s %s\n' 'the formal words found' "${formallist}"

# Evidence density, reported and never refused. `docs/decisions/.template.md`
# moves numbers out of sentences, so a record that obeys it scores bare by
# construction and a threshold would refuse the records that comply.
printf '\nevidence density: prose lines carrying no number, no code span and no link\n\n'
{ grep '^density' <<<"${prose}" || true; } | cut -f2- | sort -rn

finish
