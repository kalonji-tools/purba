#!/usr/bin/env bash
# The order of design that the rule calculates from LANDSCAPE.md.
#
#   the decision:  docs/decisions/only-a-changed-input-moves-the-order-of-design.md
#   the rule:      docs/decisions/a-design-in-depth-waits-for-what-it-assumes.md
#   the tasks:     mise run fmt:order, mise run lint:order
#
#   order-of-design.sh            writes the calculated order, and prints each step
#   order-of-design.sh --check    changes nothing, and refuses an order that differs
#
# Exits 1 when it refuses, and 2 when the landscape or its record cannot be read.
#
# The record that the order section links holds each risk's score a second time,
# beside its evidence. Both modes refuse a score that differs there, because no
# command knows which of the two is right.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"

check=0
case "${1:-}" in
  --check) check=1 ;;
  '') ;;
  *) cannot "unknown option: $1" ;;
esac

top=$(git rev-parse --show-toplevel 2>/dev/null) ||
  cannot 'the landscape can only be found inside a git repository.'
cd "${top}"

landscape=LANDSCAPE.md
[[ -r "${landscape}" ]] ||
  cannot "${landscape} cannot be read."
grep -qx '## The order of design' "${landscape}" ||
  cannot "${landscape} has no section named \"The order of design\"."

record=$(awk '
  /^## / { section = ($0 == "## The order of design") }
  section && match($0, /\]\(docs\/decisions\/[^)#]*\.md(#[^)]*)?\)/) {
    link = substr($0, RSTART + 2, RLENGTH - 3)
    sub(/#.*/, "", link)
    print link
    exit
  }
' "${landscape}")
[[ -n "${record}" && -r "${record}" ]] ||
  cannot 'the order section links no record that can be read, so no score can be compared.'

# Each line it prints starts with a tag: `unknown`, `circle`, `no-row` or `row`
# for a refusal, `step` for one step the rule takes, and `calculated` or `stated`
# for one item of each order, and `misnumbered` for an order line out of turn.
walk=$(awk '
  function trim(text) {
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", text)
    return text
  }

  # The name in "- **name.** text", and RLENGTH set past it.
  function item_name(line) {
    if (!match(line, /^- \*\*[^*]+\.\*\*/)) return ""
    return substr(line, 5, RLENGTH - 7)
  }

  function add_item(name, kind) {
    items[++item_count] = name
    kind_of[name] = kind
    if (index($0, "](")) designed[name] = 1
  }

  function known(name, kind) {
    if (!(name in kind_of) || (kind != "" && kind_of[name] != kind)) unknown[name] = 1
  }

  function compare(name, what, here, there) {
    if (here != there)
      print "row\t" name ": " what " " here " in the landscape, and " there " in the record"
  }

  # Each name of a list, with its number, in the order of the list.
  function listing(names, count, number, i, text) {
    for (i = 1; i <= count; i++) text = text ", " names[i] " " (number[names[i]] + 0)
    return " " substr(text, 3)
  }

  FNR == NR && /^## / { section = substr($0, 4); fenced = 0; next }

  FNR == NR && section == "The parts and their dependencies" {
    if (/^```/) { fenced = !fenced; next }
    if (fenced) {
      if ($2 == "--" && $(NF - 1) == "-->") {
        arrow_from[++arrow_count] = $1
        arrow_to[arrow_count] = $NF
      }
      next
    }
    name = item_name($0)
    if (name != "") add_item(name, "part")
    next
  }

  FNR == NR && section == "The cross-cutting concerns" {
    name = item_name($0)
    if (name == "") next
    add_item(name, "concern")
    rest = substr($0, RLENGTH + 1)
    sentence = substr(rest, 1, index(rest, ".") - 1)
    where = ""
    while ((at = index(sentence, ", in ")) > 0) {
      where = substr(sentence, at + 5)
      sentence = substr(sentence, at + 5)
    }
    crosses[name] = where
    next
  }

  FNR == NR && section == "The risks" {
    name = item_name($0)
    if (name == "" || !match($0, /Lands on [^;]+; probability [a-z]+, impact [a-z]+\./)) next
    split(substr($0, RSTART + 9, RLENGTH - 10), field, /; probability |, impact /)
    risks[++risk_count] = name
    lands_on[name] = field[1]
    probability[name] = field[2]
    impact[name] = field[3]
    next
  }

  FNR == NR && section == "The order of design" {
    if (match($0, /^[0-9]+\. \*\*[^*]+\.\*\*/)) {
      line = substr($0, 1, RLENGTH)
      if (line + 0 != ++stated_count) print "misnumbered\t" line
      sub(/^[0-9]+\. \*\*/, "", line)
      print "stated\t" substr(line, 1, length(line) - 3)
    }
    next
  }

  FNR == NR { next }

  # The record: the first table whose header names a risk, where it lands,
  # its probability and its impact.
  /^\|/ && !table_done {
    cells = split($0, cell, "|")
    if (!table) {
      for (i = 2; i < cells; i++) column[trim(cell[i])] = i
      if (("risk" in column) + ("lands on" in column) + ("probability" in column) \
        + ("impact" in column) == 4) {
        table = 1
        header = 1
      } else {
        delete column
      }
      next
    }
    if (header) { header = 0; next }
    risk = trim(cell[column["risk"]])
    row_lands_on[risk] = trim(cell[column["lands on"]])
    row_probability[risk] = trim(cell[column["probability"]])
    row_impact[risk] = trim(cell[column["impact"]])
    next
  }
  table && !/^\|/ { table_done = 1 }

  END {
    level["low"] = 1; level["medium"] = 2; level["high"] = 3

    for (i = 1; i <= risk_count; i++) {
      name = risks[i]
      known(lands_on[name], "")
      if (!(name in row_lands_on)) { print "no-row\t" name; continue }
      compare(name, "lands on", lands_on[name], row_lands_on[name])
      compare(name, "probability", probability[name], row_probability[name])
      compare(name, "impact", impact[name], row_impact[name])
      if (!(lands_on[name] in kind_of)) continue
      score[lands_on[name]] += level[probability[name]] * level[impact[name]]
    }

    # An item waits for each part it uses and each concern that crosses it.
    for (i = 1; i <= arrow_count; i++) {
      known(arrow_from[i], "part")
      known(arrow_to[i], "part")
      waits[arrow_from[i], arrow_to[i]] = 1
    }
    for (i = 1; i <= item_count; i++) {
      concern = items[i]
      if (kind_of[concern] != "concern" || crosses[concern] == "") continue
      where = crosses[concern]
      if (where == "every part") {
        for (j = 1; j <= item_count; j++)
          if (kind_of[items[j]] == "part") waits[items[j], concern] = 1
        continue
      }
      gsub(/ and /, ", ", where)
      count = split(where, part, ", ")
      for (j = 1; j <= count; j++) {
        known(part[j], "part")
        waits[part[j], concern] = 1
      }
    }

    for (name in unknown) print "unknown\t" name
    for (name in unknown) exit

    # Parts come before concerns on the last tie, each in the order of its lines.
    rank_count = 0
    for (pass = 1; pass <= 2; pass++)
      for (i = 1; i <= item_count; i++)
        if ((pass == 1) == (kind_of[items[i]] == "part")) ranked[++rank_count] = items[i]

    for (i = 1; i <= item_count; i++) if (items[i] in designed) done[items[i]] = 1
    step = 0
    while (1) {
      free_count = 0
      for (i = 1; i <= rank_count; i++) {
        name = ranked[i]
        if (name in done) continue
        ready = 1
        for (j = 1; j <= item_count; j++)
          if ((name, items[j]) in waits && !(items[j] in done)) ready = 0
        if (ready) free[++free_count] = name
      }
      if (free_count == 0) break

      best = 0
      for (i = 1; i <= free_count; i++) best = (score[free[i]] > best) ? score[free[i]] : best
      tied_count = 0
      for (i = 1; i <= free_count; i++) if (score[free[i]] == best) tied[++tied_count] = free[i]

      most = -1
      for (i = 1; i <= tied_count; i++) {
        waiting[tied[i]] = 0
        for (j = 1; j <= item_count; j++) {
          if (items[j] in done || !((items[j], tied[i]) in waits)) continue
          waiting[tied[i]] += score[items[j]]
        }
        if (waiting[tied[i]] > most) { most = waiting[tied[i]]; chosen = tied[i] }
      }
      leaders = 0
      for (i = 1; i <= tied_count; i++) if (waiting[tied[i]] == most) leaders++

      if (free_count == 1) {
        reason = "alone"
      } else if (tied_count == 1) {
        reason = "by score:" listing(free, free_count, score)
      } else if (leaders == 1) {
        reason = "by what waits on it:" listing(tied, tied_count, waiting)
      } else {
        reason = "by the line that comes first:" listing(tied, tied_count, waiting)
      }
      print "step\t" (++step) ". " chosen ", " reason
      print "calculated\t" chosen
      done[chosen] = 1
    }

    for (i = 1; i <= item_count; i++) if (!(items[i] in done)) print "circle\t" items[i]
  }
' "${landscape}" "${record}")

tagged() {
  sed -n "s/^$1"$'\t'"//p" <<<"${walk}"
}

joined() {
  local text
  text=$(tagged "$1")
  printf '%s' "${text//$'\n'/, }"
}

unknown=$(tagged unknown)
[[ -z "${unknown}" ]] || refuse \
  "A name is refused when it is no part or concern of the landscape, because the rule cannot \
place it. Name an item that a line of its section states." \
  "${unknown}"

circle=$(tagged circle)
[[ -z "${circle}" ]] || refuse \
  "The order of design is refused when items wait on each other in a circle, because none of \
them can come first. Remove the arrow or the crossing that closes the circle." \
  "${circle}"

no_row=$(tagged no-row)
[[ -z "${no_row}" ]] || refuse \
  "A risk is refused when the record holds no row for it, because its row holds the evidence \
for its score. Add the row to the risk table of the record that the order section links." \
  "${no_row}"

row=$(tagged row)
[[ -z "${row}" ]] || refuse \
  "A risk is refused when its score or the item it lands on differs from its row in the record, \
because the row holds the evidence for that score. Rewrite whichever of the two is wrong." \
  "${row}"

[[ -z "${unknown}" && -z "${circle}" ]] || finish

steps=$(tagged step)
calculated=$(joined calculated)
stated=$(joined stated)
misnumbered=$(tagged misnumbered)
differs=()
[[ "${stated}" == "${calculated}" ]] ||
  differs+=("stated:     ${stated}" "calculated: ${calculated}")
[[ -z "${misnumbered}" ]] ||
  differs+=("numbered out of turn:" "${misnumbered}")

if [[ "${check}" -eq 1 ]]; then
  [[ ${#differs[@]} -eq 0 ]] || refuse \
    "The order of design is refused when it differs from the order the rule calculates, because \
two people must get one order from the same landscape. Run mise run fmt:order." \
    "${differs[@]}" "${steps}"
  finish
fi

printf '%s\n' "${steps}"
if [[ ${#differs[@]} -gt 0 ]]; then
  [[ -n "${stated}" ]] ||
    cannot 'the order section holds no numbered list to rewrite.'
  lines=$(tagged calculated | awk '{ printf "%d. **%s.**\n", NR, $0 }')
  written=$(awk -v lines="${lines}" '
    /^## / { section = ($0 == "## The order of design") }
    section && /^[0-9]+\. \*\*/ { if (!written && lines != "") print lines; written = 1; next }
    { print }
  ' "${landscape}")
  printf '%s\n' "${written}" >"${landscape}"
fi
finish
