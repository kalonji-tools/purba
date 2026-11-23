#!/usr/bin/env bash
# Each use between two parts, in the manifests of the crates and in the
# sequence of one run, that the parts diagram of LANDSCAPE.md does not draw.
# Each part that names pyo3 when the line of the Rust/Python seam does not name it.
# Each use of the crate of a concern by a part that the concern's line does not name.
# Each part that the crate of a concern uses.
#
#   the decision:  docs/decisions/each-part-is-a-crate.md
#   the task:      mise run lint:wiring
#
#   check-wiring.sh
#
# Exits 1 when it refuses, and 2 when the landscape or a manifest cannot be read.
#
# The crate `purba` builds the extension module and is not a part, so the check
# skips it. An arrow reads only as `a -- "label" --> b`, as order-of-design.sh
# reads it, and a message only as `->>` or `-->>`.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"

top=$(git rev-parse --show-toplevel 2>/dev/null) ||
  cannot 'the wiring can only be found inside a git repository.'
cd "${top}"

landscape=LANDSCAPE.md
section='The parts and their dependencies'
[[ -r "${landscape}" ]] ||
  cannot "${landscape} cannot be read."
grep -qx "## ${section}" "${landscape}" ||
  cannot "${landscape} has no section named \"${section}\"."
concerns='The cross-cutting concerns'
grep -qx "## ${concerns}" "${landscape}" ||
  cannot "${landscape} has no section named \"${concerns}\"."

# The parts after the last ", in " of the first sentence of the seam's line, one
# to a line.
seam=$(awk -v section="## ${concerns}" -v start='- **the Rust/Python seam.** ' '
  /^## / { inside = ($0 == section); next }
  inside && index($0, start) == 1 {
    found = 1
    line = substr($0, length(start) + 1)
    sub(/\. .*/, "", line)
    sub(/\.$/, "", line)
    if (!sub(/.*, in /, "", line)) next
    gsub(/ and /, ", ", line)
    count = split(line, parts, ", ")
    for (i = 1; i <= count; i++) print parts[i]
  }
  END { exit !found }
' "${landscape}") ||
  cannot "${landscape} has no line for the Rust/Python seam."

# Each concern that names its crate with "Its crate is `purba-<x>`.": the crate,
# then the parts after the last ", in " of the line's first sentence. The awk
# splits them as it splits the seam's parts. "every part" stands for each part.
concern_crates=$(awk -v section="## ${concerns}" '
  /^## / { inside = ($0 == section); next }
  !inside || !match($0, /^- \*\*[^*]+\.\*\* /) { next }
  {
    rest = substr($0, RLENGTH + 1)
    if (!match(rest, /Its crate is `purba-[a-z-]+`/)) next
    crate = substr(rest, RSTART + 14, RLENGTH - 15)
    sub(/\. .*/, "", rest)
    sub(/\.$/, "", rest)
    if (!sub(/.*, in /, "", rest)) rest = ""
    gsub(/ and /, ", ", rest)
    print crate "\t" rest
  }
' "${landscape}") ||
  cannot "${landscape} cannot be read."
crates=$(cargo metadata --no-deps --format-version 1 --manifest-path Cargo.toml) ||
  cannot 'Cargo.toml cannot be read.'

# Each line it prints starts with a tag: `part` for a part, `arrow` for an arrow
# of the parts diagram, `outside` for an actor or `_main` in the sequence, and
# `call` or `answer` for a message of the sequence, followed by its line.
drawn=$(awk -v section="## ${section}" '
  /^## / { inside = ($0 == section); fence = ""; next }
  !inside { next }
  /^```/ { fence = (fence == "" ? "open" : ""); next }
  fence == "" && match($0, /^- \*\*[^*]+\.\*\*/) {
    print "part\t" substr($0, 5, RLENGTH - 7)
    next
  }
  fence == "open" { fence = $1; next }
  fence == "flowchart" && $2 == "--" && $(NF - 1) == "-->" {
    print "arrow\t" $1 "\t" $NF
    next
  }
  fence != "sequenceDiagram" { next }
  $1 == "actor" || ($1 == "participant" && $NF == "_main") {
    print "outside\t" $2
    next
  }
  $2 == "->>" || $2 == "-->>" {
    to = $3
    sub(/:$/, "", to)
    line = $0
    sub(/^[[:space:]]+/, "", line)
    print ($2 == "->>" ? "call" : "answer") "\t" $1 "\t" to "\t" line
  }
' "${landscape}") ||
  cannot "${landscape} cannot be read."

# shellcheck disable=SC2016 # jq's own variables, not the shell's
findings=$(jq -nr --arg top "${top}/" --arg drawn "${drawn}" --arg seam "${seam}" \
  --arg concern_crates "${concern_crates}" --argjson crates "${crates}" '
  ($drawn | split("\n") | map(select(. != "") | split("\t"))) as $lines
  | ($concern_crates | split("\n") | map(select(. != "") | split("\t")
      | {key: .[0], value: (.[1] // "" | split(", "))}) | from_entries) as $concern
  | ($seam | split("\n") | map(select(. != ""))) as $seam
  | [$lines[] | select(.[0] == "part") | .[1]] as $parts
  | [$lines[] | select(.[0] == "arrow") | "\(.[1]) \(.[2])"] as $arrows
  | [$lines[] | select(.[0] == "outside") | .[1]] as $outside
  | [$crates.packages[] | select(.name | startswith("purba-"))
      | {part: (.name | ltrimstr("purba-")), name,
        manifest: (.manifest_path | ltrimstr($top)),
        pyo3: any(.dependencies[]; .name == "pyo3" and .kind != "dev"),
        uses: [.dependencies[].name | select(startswith("purba-")) | ltrimstr("purba-")]}
    ] as $members
  | ($members | map({key: .part, value: .}) | from_entries) as $crate
  | [
      ($members[] | select((.part | IN($parts[]) | not) and $concern[.name] == null)
        | "no-part-crate\t\(.name), in \(.manifest)"),
      ($members[] | select(.part | IN($parts[])) | . as $member | .uses[]
        | select($concern["purba-\(.)"] == null)
        | select("\($member.part) \(.)" | IN($arrows[]) | not)
        | "undrawn\t\($member.part) uses \(.), in \($member.manifest)"),
      ($members[] | select(.part | IN($parts[])) | . as $member | .uses[]
        | . as $used | $concern["purba-\(.)"] | select(. != null)
        | select(IN(.[]; "every part", $member.part) | not)
        | "concern-unnamed\t\($member.part) uses purba-\($used), in \($member.manifest)"),
      ($members[] | select($concern[.name] != null and .uses != [])
        | "concern-uses-part\t\(.name) uses \(.uses | join(", ")), in \(.manifest)"),
      ($arrows[] | split(" ") as [$from, $to]
        | select($crate[$from] != null and $crate[$to] != null)
        | select($to | IN($crate[$from].uses[]) | not)
        | "unused\t\($from) --> \($to), and \($crate[$from].manifest) names no purba-\($to)"),
      ($members[] | select(.pyo3) | select(.part | IN($seam[]) | not)
        | "off-seam\t\(.name) names pyo3, in \(.manifest)"),
      ($lines[] | select(.[0] == "call" or .[0] == "answer") | . as $message
        | if any(.[1], .[2]; IN($parts[], $outside[]) | not) then
            "no-part-end\t\(.[3])"
          elif (.[1] | IN($parts[])) and (.[2] | IN($parts[])) then
            (if .[0] == "call" then "\(.[1]) \(.[2])" else "\(.[2]) \(.[1])" end)
            | select(IN($arrows[]) | not)
            | "\($message[0])\t\($message[3])"
          else empty end)
    ]
  | .[]
') ||
  cannot 'the wiring cannot be compared.'

refusals() {
  grep "^$1"$'\t' <<<"${findings}" | cut -f2- || true
}

found=$(refusals no-part-crate)
[[ -z "${found}" ]] || refuse \
  "A crate named purba-<part> is refused when <part> is no part of the diagram, because \
the check reads the arrows of a part only. Name the part in the landscape, or rename the crate." \
  "${found}"

found=$(refusals undrawn)
[[ -z "${found}" ]] || refuse \
  "A part may use only a part that the parts diagram draws an arrow to, because the \
diagram is the only wiring. Draw the arrow, or remove the dependency." \
  "${found}"

found=$(refusals concern-unnamed)
[[ -z "${found}" ]] || refuse \
  "A part may use the crate of a concern only when the concern's line names the part, because \
no arrow of the parts diagram draws that use. Name the part in the concern's line, or remove the \
dependency." \
  "${found}"

found=$(refusals concern-uses-part)
[[ -z "${found}" ]] || refuse \
  "The crate of a concern uses no part, because each part may use it. Remove the dependency." \
  "${found}"

found=$(refusals unused)
[[ -z "${found}" ]] || refuse \
  "An arrow between two crates is refused when the manifest does not use it, because the \
diagram states what purba uses now. Remove the arrow, or add the dependency." \
  "${found}"

found=$(refusals call)
[[ -z "${found}" ]] || refuse \
  "A call in the sequence of one run must go along an arrow of the parts diagram, because \
the diagram is the only wiring. Draw the arrow, or change the message." \
  "${found}"

found=$(refusals answer)
[[ -z "${found}" ]] || refuse \
  "An answer in the sequence of one run must go back along an arrow of the parts diagram, \
because the diagram is the only wiring. Draw the arrow, or change the message." \
  "${found}"

found=$(refusals no-part-end)
[[ -z "${found}" ]] || refuse \
  "Each end of a message in the sequence of one run must be a part, an actor or _main, \
because the check reads the arrows of a part only. Name the part in the landscape, or correct \
the name." \
  "${found}"

found=$(refusals off-seam)
[[ -z "${found}" ]] || refuse \
  "Only a part that the line of the Rust/Python seam names may name pyo3, because the other \
parts pass data. Name the part in the seam's line, or remove the dependency." \
  "${found}"

finish
