#!/usr/bin/env bash
# The decision-record rules a command can decide, in each record and in the
# comment of the template.
#
#   the decision:  docs/decisions/a-record-is-rewritten-not-amended.md
#   the task:      mise run lint:records
#
#   check-records.sh [directory]
#
#   PURBA_CHECK_PROSE   a built check-prose, run in place of `cargo run`, when set
#
# Exits 1 when a record or the comment of the template breaks a rule, and 2
# when this script cannot decide.
#
# It reports every rule before it exits, because a writer fixing one refusal
# should not have to run it again to find the next.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"
# shellcheck source=scripts/records.sh
. "$(dirname "$0")/records.sh"

dir=${1:-docs/decisions}

# A leading dot does not match `*`, so the template is excluded by the glob
# rather than by a name test.
shopt -s nullglob
records=("${dir}"/*.md)

[[ ${#records[@]} -gt 0 ]] || cannot "no record found in ${dir}."

scratch=$(mktemp -d)
trap 'rm -rf "${scratch}"' EXIT

# A cargo script reads the prose rules, the section order, the Downside, the
# Confirmation, and the comment, issue-number and citation rules. It reads
# through CommonMark, so a line in a code block is not prose. It prints one
# tagged line for each finding, and the table below gives each tag its summary.
#
#   the decision:  docs/decisions/an-artifact-holds-the-minimum-that-conveys-its-point.md
here=$(cd "$(dirname "$0")" && pwd)
read_prose=(cargo -Zscript --config "resolver.lockfile-path=\"${here}/check-prose/Cargo.lock\""
  run --quiet --release --locked --manifest-path "${here}/check-prose/check-prose.rs"
  --target-dir "${here}/../target/scripts" -- "${here}/check-prose/participles.txt")
#   the decision:  docs/decisions/one-workflow-runs-every-gate-of-a-pull-request.md
[[ -z "${PURBA_CHECK_PROSE:-}" ]] ||
  read_prose=("${PURBA_CHECK_PROSE}" "${here}/check-prose/participles.txt")
prose=$("${read_prose[@]}" "${records[@]}") ||
  cannot 'the prose of a record cannot be read.'

# The template is the register of the records, and its comment obeys their
# prose rules. A second run reads the comment, with each other line blank, so
# its sentences join no tally. The rules that read the parts of a record do
# not bind it.
#
#   the decision:  docs/decisions/a-register-belongs-to-one-location.md
template=${dir}/.template.md
unread="the comment of ${template} cannot be read, so the rules a record obeys cannot be \
checked in it. Its first line must be \`<!--\` and a later line must be \`-->\`, and the \
two markers each stand alone on a line."
[[ -e "${template}" ]] ||
  cannot "the comment of ${template} cannot be read, because the file is missing."
[[ -r "${template}" ]] ||
  cannot "the comment of ${template} cannot be read, because the file cannot be opened."
# template_comment is one awk command, so set -e has nothing inside it to stop.
# shellcheck disable=SC2310
template_comment "${template}" >"${scratch}/template.md" || cannot "${unread}"
register=$("${read_prose[@]}" "${scratch}/template.md") ||
  cannot 'the prose of the template cannot be read.'
parts=$'^(sections|comment|preamble|counted|listless|leadins|unwired|tally|density)\t'
while IFS= read -r line; do
  [[ -z "${line}" || "${line}" =~ ${parts} ]] ||
    prose+=$'\n'"${line//"${scratch}/template.md"/"${template}"}"
done <<<"${register}"

# A bare record name in the comment is a file beside the records. A path, an
# upper-case name such as CONTEXT.md, and a name joined to an underscore or to a
# second extension are no record names. A full stop may follow a name.
name='(^|[^/._[:alnum:]-])([a-z0-9][a-z0-9-]*\.md)(\.([^[:alnum:]]|$)|[^./_[:alnum:]-]|$)'
found=()
declare -A named=()
number=0
while IFS= read -r line; do
  number=$((number + 1))
  # The characters after a name can sit before the next one, as the `,` of
  # `a.md,b.md` does, so the scan starts again at them. A name at the end of the
  # line ends the scan. A line refuses each missing name once.
  while [[ "${line}" =~ ${name} ]]; do
    entry="${template}:${number}: ${BASH_REMATCH[2]}"
    after=${BASH_REMATCH[3]}
    whole=${BASH_REMATCH[0]}
    if [[ ! -f "${dir}/${BASH_REMATCH[2]}" && -z "${named[${entry}]:-}" ]]; then
      found+=("${entry}")
      named[${entry}]=1
    fi
    [[ -n "${after}" ]] || break
    line=${after}${line#*"${whole}"}
  done
done <"${scratch}/template.md"
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A record that the template's comment names is a file beside the records, because a writer \
who follows the comment opens it. A record keeps the file name it was born with, so only a \
deleted record leaves its name dead." \
  "${found[@]}"

# Each tag the cargo script prints, whether it refuses or is reported, and its
# summary. The output follows the order of the rows. The summary of a refusal
# stays here, where report.sh refuses one that names a record.
rules=(
  sections refuse "A record carries four sections, in order: Context and Problem Statement, \
Considered Options, Decision Outcome, Confirmation. They are a public Markdown convention cut to \
its minimum, and Confirmation keeps a record from going stale. Consequences is optional and sits \
between the outcome and the confirmation."

  comment refuse "A record reads as current state, so it carries no comment addressed to a \
reviewer. The template carries the drafting instructions, and a leading dot keeps it out of this \
glob."

  issue refuse "A record states what is true and an issue states what happened, so record prose \
carries no issue number. Title the link with the question its issue asks."

  emdash refuse "A record carries no em-dash, because an em-dash joins two ideas in one sentence. \
Use a full stop, a colon, a comma or a list. A code span and a fenced block are exempt, because \
quoting one is quoting an artifact."

  bold refuse "Bold opens a sentence and never sits inside one. A lead-in is what bold is for, and \
whether a bolded phrase is one is read by a reviewer rather than decided here."

  wrap refuse "A sentence stays on one line. A record is rewritten in place, and a wrapped \
paragraph reflows on a one word edit and buries the change in the diff. The line named is where \
the sentence breaks."

  long refuse "A sentence in descriptive text runs to 25 words, a limit borrowed from Simplified \
Technical English. A link counts as one word and so does a code span, because a reader reads each \
as one thing."

  step refuse "A sentence in a numbered list runs to 20 words, a limit borrowed from Simplified \
Technical English for a procedure. A record writes a procedure as a numbered list, so the limit \
reaches each of its items."

  para refuse "A paragraph runs to six sentences, a limit borrowed from Simplified Technical \
English. The line named is where the seventh lands, and a blank line splits the paragraph."

  ing refuse "An -ing form is a technical noun here and never a verb, a rule borrowed from \
Simplified Technical English. A gerund after a form of be, a preposition or a conjunction becomes \
a finite clause."

  # Harper's tagger decides the tense rule.
  #
  #   the decision:  docs/decisions/a-part-of-speech-tagger-decides-the-tense-rule.md
  tense refuse "A record uses the simple tenses, a rule borrowed from Simplified Technical \
English. A modal with the bare verb is one of them, so must be run stands and has run does not."

  # The figure rules a command can decide.
  #
  #   the decision:  docs/decisions/a-figure-in-a-record-is-a-fence-github-renders.md
  math refuse "A formula in a record is a math fence, so \$ math is refused outside one. A record \
writes each kind of figure in one form."

  icon refuse "A mermaid fence in a record draws no icon. GitHub registers no icon pack and loads \
no FontAwesome, so an icon draws nothing and raises no error. The built-in icons of an \
architecture diagram are the exception: cloud, database, disk, internet and server."

  preamble refuse "A Downside label stands on its own line. Its costs are the list beneath it, so \
a preamble is a sentence the reader carries while reading them."

  counted refuse "A Downside label never counts its costs. A count is one more thing to keep true, \
and one record counted four costs where three rendered."

  listless refuse "A record states its costs as a list under a Downside label, because a decision \
without its cost is advocacy, not a record. A record with no label has no list either, so a \
missing Downside is refused here."

  leadins refuse "Bold marks every Downside lead-in, one for each cost. Fewer lead-ins than costs \
leaves a cost unmarked, and more means a cost opens mid-line and does not render."

  adr refuse "A number cannot be checked against the record it names, so a record cites another \
by its proposition."

  unwired report "a Confirmation that admits an unwired gate and names no issue"
)

# A tag with no rule would be dropped, and the record that breaks it would pass.
declare -A known=([tally]=1 [density]=1)
for ((i = 0; i < ${#rules[@]}; i += 3)); do
  known[${rules[i]}]=1
done
while IFS=$'\t' read -r tag _; do
  [[ -z "${tag}" || -n "${known[${tag}]:-}" ]] ||
    cannot "the prose check printed the tag ${tag}, and no rule here holds it."
done <<<"${prose}"

# A record with no Decision Outcome heading is refused for its sections.
#
#   the decision:  docs/decisions/a-record-names-every-location-where-it-applies.md
found=()
for f in "${records[@]}"; do
  span=$(reach_lines "${f}")
  [[ -z "${span}" || "${span}" == *' reach' ]] || found+=("${f}:${span%% *}")
done
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A Decision Outcome opens with a reach: a **Reach:** paragraph that names every location where \
a change makes the record apply, and holds nothing else. A record with no reach names no \
location, so no change can be matched to it, and the prose rules do not read a reach. Write a \
path as a gitattributes pattern in a code span, and a location with no path as its glossary \
word, linked to its CONTEXT.md entry." \
  "${found[@]}"

# A record names its areas in the paragraph after its reach. The landscape sits
# two folders above the records, as each record's own link to it says, and it
# holds every area but four.
#
#   the decision:
# docs/decisions/a-record-names-its-area-and-docs-readme-lists-the-records-by-area.md
landscape=${dir}/../../LANDSCAPE.md
names=''
[[ ! -r "${landscape}" ]] || names=$(area_names "${landscape}")
[[ -n "${names}" ]] ||
  cannot "${landscape} names no part or concern, so no area can be checked."
declare -A area=()
while IFS= read -r name; do
  area[${name}]=1
done <<<"${names}"

found=()
for f in "${records[@]}"; do
  area_read "${f}" at areas
  [[ -n "${at}" ]] || continue
  if [[ -z "${areas}" ]]; then
    found+=("${f}:${at}")
    continue
  fi
  while IFS= read -r name; do
    [[ -n "${area[${name}]:-}" ]] || found+=("${f}:${at}: ${name}")
  done <<<"${areas}"
done
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A record names each area it decides something in, in an \`**Area:**\` paragraph after its \
reach. docs/README.md is written from these lines, so a record with no line is missing from it. An \
area is design, workflow, writing, tooling, or the name of a part or a concern in LANDSCAPE.md." \
  "${found[@]}"

# Considered Options holds only what lost, so an option there carries no mark.
#
#   the decision:
# docs/decisions/the-decision-sentence-is-the-winner-and-considered-options-holds-only-what-lost.md
chosen=()
rejected=()
for f in "${records[@]}"; do
  marks=$(option_marks "${f}")
  while read -r number word; do
    if [[ "${word}" == Chosen ]]; then
      chosen+=("${f}:${number}")
    elif [[ -n "${word}" ]]; then
      rejected+=("${f}:${number}")
    fi
  done <<<"${marks}"
done
[[ ${#chosen[@]} -eq 0 ]] || refuse \
  "Considered Options holds only what lost, so no option there is marked Chosen. The bold \
decision sentence after the area is the winner, and its reason follows it. Each other winner that \
Decision Outcome does not already state is a list line under that sentence." \
  "${chosen[@]}"
# The decision sentence is the winner, so it opens the paragraph after the
# area, on one bold line that ends a sentence. A code span may hold `**`, so
# each one is read as one word first. A record with no area line is refused
# above, and has no decision sentence to read here.
tick=$'\x60'
# A closing emphasis may come before the stop, and a closing quote or
# parenthesis after it.
decision='^\*\*([^*]|\*[^*])+\*?[.?!]["'"'"')]*\*\*$'
found=()
for f in "${records[@]}"; do
  decision_read "${f}" at line
  [[ -n "${at}" ]] || continue
  line=${line%"${line##*[![:space:]]}"}
  while [[ "${line}" =~ ${tick}[^${tick}]+${tick} ]]; do
    line=${line/"${BASH_REMATCH[0]}"/x}
  done
  [[ "${line}" =~ ${decision} ]] || found+=("${f}:${at}")
done
[[ ${#found[@]} -eq 0 ]] || refuse \
  "A Decision Outcome opens, after its area, with the decision in one line that \`**\` wraps \
and whose words end with \`.\`, \`?\` or \`!\`. The decision sentence is the record's winner, so \
a reader finds the decision in one place. The line named is where the decision sentence stands." \
  "${found[@]}"
[[ ${#rejected[@]} -eq 0 ]] || refuse \
  "A lost option states why it lost, with no mark, because Rejected marks nothing in a list of \
losers. \`mise run fmt:records\` strips Rejected., and the author rewrites any other form by \
hand." \
  "${rejected[@]}"

for ((i = 0; i < ${#rules[@]}; i += 3)); do
  mapfile -t found < <(grep "^${rules[i]}"$'\t' <<<"${prose}" | cut -f2- || true)
  [[ ${#found[@]} -gt 0 ]] || continue
  if [[ "${rules[i + 1]}" == refuse ]]; then
    refuse "${rules[i + 2]}" "${found[@]}"
  else
    printf '\n%s\n\n' "${rules[i + 2]}"
    printf '  %s\n' "${found[@]}"
  fi
done

# Each pattern a reach writes in a code span is matched against the tracked
# files. Outside a git repository no file is tracked, and nothing is
# matched.
if top=$(git rev-parse --show-toplevel 2>/dev/null); then
  # git writes NUL-separated output, which a shell variable cannot hold.
  git -C "${top}" ls-files -z >"${scratch}/files" ||
    cannot "git cannot list the tracked files."
  reach_hits "${scratch}" "${scratch}/files" "${records[@]}" >"${scratch}/matched"

  declare -A matched=()
  while IFS= read -r -d '' f && IFS= read -r -d '' pattern && IFS= read -r -d '' _; do
    matched["${f}"$'\n'"${pattern}"]=1
  done <"${scratch}/matched"

  found=()
  for f in "${records[@]}"; do
    patterns=$(reach_patterns "${f}")
    [[ -n "${patterns}" ]] || continue
    while IFS= read -r pattern; do
      [[ -n "${matched["${f}"$'\n'"${pattern}"]:-}" ]] || found+=("${f}: ${pattern}")
    done <<<"${patterns}"
  done
  if [[ ${#found[@]} -gt 0 ]]; then
    printf '\na pattern in a reach that matches no tracked file\n\n'
    printf '  %s\n' "${found[@]}"
  fi
fi

# Three borrowed rules a command cannot decide, reported and never
# refused. The active voice rule admits the passive where a sentence cannot
# name who acts, which is a judgement, and an article left out is not
# decidable at all: the count below says how many are present, never how many
# are missing.
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
