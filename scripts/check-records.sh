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

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"

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

# The blank line keeps one refusal apart from the next in a terminal.
refuse() {
  printf '\n' >&2
  report "$1" "$(printf '  %s\n' "${@:2}")"
  broken=1
}

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

# The two prose rules `docs/decisions/.template.md` states.
#
#   the decision:  docs/decisions/an-artifact-holds-the-minimum-that-conveys-its-point.md
#
# `**` opens bold only where it flanks, which is a left-flanking delimiter run.
# https://spec.commonmark.org/0.31.2/#left-flanking-delimiter-run
# Without that test an unbackticked `.github/**` and `/scripts/**` on one line
# pair into a bold run, and masking code spans does not reach it.
prose=$(awk '
  BEGIN {
    limit = 25
    parlimit = 6

    # A word keeps its apostrophe and its hyphen. The separator is built
    # rather than written, because this awk program is inside a quoted
    # string and an apostrophe would close it.
    wordsep = "[^A-Za-z" sprintf("%c", 39) "-]+"

    # An `-ing` word the rule admits, because it is a technical noun rather
    # than a verb form.
    split("setting settings heading headings listing listings mapping " \
          "mappings warning warnings wording tooling tracking logging " \
          "string strings thing things nothing something anything " \
          "everything during according including being morning evening " \
          "spring ceiling meaning rating ratings casing padding wrapping", w, " ")
    for (i in w) ingok[w[i]] = 1

    split("is are was were be been being am", w, " ")
    for (i in w) beform[w[i]] = 1

    split("by for of without before after while when on in about from than " \
          "through against into onto over under", w, " ")
    for (i in w) prep[w[i]] = 1

    split("not never already also still now then always often again " \
          "just therefore once since ever twice yet", w, " ")
    for (i in w) adverb[w[i]] = 1

    # A participle that does not end in `-ed`.
    split("been begun bound bought broken brought built burnt caught " \
          "chosen cost cut dealt done drawn driven eaten fallen felt " \
          "fought found forgotten given gone grown heard held kept laid " \
          "led left lent lost made meant met paid put run said seen sold " \
          "sent set shown shut sung slept spoken spent stood taken taught " \
          "told thought thrown understood won written forbidden hidden " \
          "risen torn worn proven shaken stuck struck sworn read beaten " \
          "frozen", w, " ")
    for (i in w) irreg[w[i]] = 1


    # A formal word the standard replaces with a plain one. The list holds
    # only words with no second sense here: `required` and `per` are left out
    # because GitHub names a required check and a rate per hour.
    split("utilize utilise utilization commence commences commenced " \
          "terminate terminates terminated endeavour endeavor ascertain " \
          "aforementioned notwithstanding whilst amongst heretofore " \
          "thereof herein pursuant facilitate facilitates expedite", w, " ")
    for (i in w) formal[w[i]] = 1
  }

  # A code span becomes \001. Splitting on the backtick puts every span in an
  # even field, which is what a regex cannot do: it pairs a closing backtick
  # with a later opening one.
  function mask(line,   part, n, i, out) {
    n = split(line, part, "`")
    out = part[1]
    for (i = 2; i <= n; i++) out = out (i % 2 == 0 ? "\001" : part[i])
    return out
  }

    # A link counts as one word, and its title is not read as prose. The rule above
  # refuses an issue number in record prose and asks for the question its issue
  # asks, so the title is owed rather than chosen. Charging its words to the
  # sentence would refuse the sentence that obeys that rule.
  function strip_link(s) {
    gsub(/\[[^]]*\]\([^)]*\)/, "\001", s)
    return s
  }

  # A code span counts as one word, because a reader reads it as one thing. The
  # mask leaves it as one character carrying no letter, so it becomes the same
  # word the detectors below put in its place.
  function words(s,   n, i, arr, c) {
    gsub(/\001/, " codespan ", s)
    n = split(s, arr, /[[:space:]]+/)
    c = 0
    for (i = 1; i <= n; i++) if (arr[i] ~ /[A-Za-z0-9]/) c++
    return c
  }

  # A boundary is `.`, `!` or `?` then whitespace. A version number and a bare
  # filename keep their full stop, because neither puts a space after it.
  #
  # A lowercase word is not a continuation. The corpus holds no abbreviation
  # and no initial, and it opens a sentence with a lowercase name: `purba`,
  # `mise`, `zig` and `devenv`. Reading those as continuations joined two
  # sentences and refused the pair for the length of both.
  function split_sentences(unit, sent,   n, cur, head, tail, rest) {
    n = 0
    cur = ""
    rest = unit
    while (match(rest, /[.!?][*")\]]*[[:space:]]+/)) {
      head = substr(rest, 1, RSTART + RLENGTH - 1)
      tail = substr(rest, RSTART + RLENGTH)
      n++
      sent[n] = cur head
      cur = ""
      rest = tail
    }
    if (rest ~ /[^[:space:]]/) {
      n++
      sent[n] = cur rest
    }
    return n
  }

  # Three letters or fewer is not a participle, which is what keeps `red` out.
  function is_participle(w) {
    if (w in irreg) return 1
    return (w ~ /ed$/ && length(w) > 3)
  }

  function is_adverb(w) {
    return (w in adverb) || w ~ /ly$/
  }

  # The word before, skipping every adverb.
  function prior(arr, i,   p) {
    while (--i >= 1) {
      p = tolower(arr[i])
      if (! is_adverb(p)) return p
    }
    return ""
  }

  # A masked code span becomes a word and never a gap. Dropping it makes the
  # words on either side adjacent, and `over CODE, checking` then reads as a
  # preposition with a gerund nobody wrote.
  function ing_fault(sent,   n, i, arr, lw, p) {
    gsub(/\001/, " codespan ", sent)
    n = split(sent, arr, wordsep)
    for (i = 1; i <= n; i++) {
      lw = tolower(arr[i])
      # A hyphenated word is a compound adjective and never a verb form, so
      # `load-bearing` after `is` is not the progressive it looks like.
      if (lw ~ /-/) continue
      if (lw !~ /ing$/ || length(lw) < 5 || lw in ingok) continue
      p = prior(arr, i)
      if (p in beform || p in prep) return arr[i]
    }
    return ""
  }

  # `has`, `have` or `had` with a participle, and `having` with one. A modal
  # with `be` is the infinitive, which the standard admits, so `must be run`
  # is not a fault.
  function tense_fault(sent,   n, i, arr, lw, p) {
    gsub(/\001/, " codespan ", sent)
    n = split(sent, arr, wordsep)
    for (i = 1; i <= n; i++) {
      lw = tolower(arr[i])
      if (! is_participle(lw) && lw != "been") continue
      p = prior(arr, i)
      if (p == "has" || p == "have" || p == "had" || p == "having") return p " " lw
    }
    return ""
  }

  # Everything the report reads, in one pass over the sentence. The passive
  # voice is reported and never refused, because the rule admits the passive
  # where the agent is unknown and no command decides that.
  function tally(sent,   n, i, arr, lw, p, passive) {
    gsub(/\001/, " codespan ", sent)
    n = split(sent, arr, wordsep)
    passive = 0
    for (i = 1; i <= n; i++) {
      lw = tolower(arr[i])
      if (lw == "a" || lw == "an" || lw == "the") arts++
      if (lw in formal) { formals++; seenformal[lw] = 1 }
      if (lw ~ /[A-Za-z]/) wordcount++
      if (passive || ! is_participle(lw)) continue
      p = prior(arr, i)
      if (p in beform) passive = 1
    }
    return passive
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
  FNR == 1 { fence = 0; psent = 0; preported = 0 }

  /^[[:space:]]*```/ { fence = ! fence; psent = 0; preported = 0; next }
  fence { next }
  # A heading is not a sentence, and it closes the paragraph above it. The
  # evidence-density pass below skips one too.
  /^#/ { psent = 0; preported = 0; next }
  {
    masked = mask($0)
    # A unit is a line, or a table cell once the row is split on the pipes that
    # are not inside a code span.
    units = 1
    cell[1] = masked
    if ($0 ~ /^\|/) units = split(masked, cell, /\|/)

    # The em-dash and the bold rules read a table cell. The length rules do
    # not: a cell is a cell, and the template puts numbers in a table on
    # purpose, so a length rule there would push them back into prose.
    table_row = ($0 ~ /^\|/)

    # A sentence in a list item is still a sentence, so a Downside cost cannot
    # dodge the length rule by being a bullet.
    text_line = ($0 ~ /[^[:space:]]/ && ! table_row && $0 !~ /^<!--/)

    # A paragraph is narrower: a run of lines at column zero. A blank line, a
    # table, a list item, a quotation and a heading all close one.
    prose_line = (text_line && $0 !~ /^[[:space:]]/ && $0 !~ /^[-*+] / \
                  && $0 !~ /^[0-9]+\. / && $0 !~ /^>/)
    if (! prose_line) { psent = 0; preported = 0 }

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

    if (text_line) {
      line = strip_link(masked)
      sub(/^[[:space:]]+/, "", line)
      sub(/^([-*+]|[0-9]+\.)[[:space:]]+/, "", line)
      ns = split_sentences(line, sent)
      if (prose_line) psent += ns
      for (s = 1; s <= ns; s++) {
        n = words(sent[s])
        if (n > limit && ! seen["l" FILENAME ":" FNR]++) \
          printf "long\t%s:%d: %d words\n", FILENAME, FNR, n
        hit = ing_fault(sent[s])
        if (hit != "" && ! seen["i" FILENAME ":" FNR]++) \
          printf "ing\t%s:%d: %s\n", FILENAME, FNR, hit
        hit = tense_fault(sent[s])
        if (hit != "" && ! seen["t" FILENAME ":" FNR]++) \
          printf "tense\t%s:%d: %s\n", FILENAME, FNR, hit

        sentences++
        if (tally(sent[s])) passives++
      }
    }

    if (prose_line && psent > parlimit && ! preported++) \
      printf "para\t%s:%d: %d sentences\n", FILENAME, FNR, psent
  }

  END {
    list = ""
    for (f in seenformal) list = list (list == "" ? "" : " ") f
    printf "tally\t%d\t%d\t%d\t%d\t%d\t%s\n", \
      sentences, passives, arts, formals, wordcount, list
  }
' "${records[@]}")

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
Technical English. A gerund after a form of be or after a preposition becomes a finite clause." \
  "${found[@]}"

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
  "A Confirmation that says a gate is unwired names the issue that will wire it, so the promise \
has an owner." \
  "${found[@]}"

# The three borrowed rules a command cannot decide, reported and never
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
    # Same reason as the report above: a corpus with no prose line is
    # refused already, and dividing by its zero would abort this one.
    if (lines == 0)
      printf "all records  no prose line to read\n"
    else
      printf "all records  %.1f%%  %d of %d\n", 100 * barelines / lines, barelines, lines
  }
' "${records[@]}" | sort -rn

exit "${broken}"
