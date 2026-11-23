#!/usr/bin/env bash
# Whether each line of .config/needs.toml states a need of a part or concern of
# the landscape, and whether each entry of .config/shortlists.toml has its need's line.
#
#   the decision:  docs/decisions/a-forecast-finds-the-crates-for-each-stated-need.md
#   the task:      mise run lint:needs
#
#   check-needs.sh
#
# It asks no server anything: it fetches no link, and no keyword from crates.io.
#
# Exits 1 when it refuses, and 2 when .config/needs.toml, .config/shortlists.toml or the
# parts and concerns of the landscape cannot be read.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"

top=$(git rev-parse --show-toplevel 2>/dev/null) ||
  cannot 'the needs can only be found inside a git repository.'
cd "${top}"

needs=$(yq -p toml -o json -I0 . .config/needs.toml) ||
  cannot '.config/needs.toml cannot be read.'
shortlists=$(yq -p toml -o json -I0 . .config/shortlists.toml) ||
  cannot '.config/shortlists.toml cannot be read.'

landscape=LANDSCAPE.md
[[ -r "${landscape}" ]] ||
  cannot "${landscape} cannot be read."
parts_section='## The parts and their dependencies'
concerns_section='## The cross-cutting concerns'
grep -qxF "${parts_section}" "${landscape}" || grep -qxF "${concerns_section}" "${landscape}" ||
  cannot "${landscape} has neither a section named \"${parts_section#'## '}\" nor one named \
\"${concerns_section#'## '}\"."

# The name in each line "- **name.**" of the parts and of the concerns.
parts=$(awk -v parts="${parts_section}" -v concerns="${concerns_section}" '
  /^## / { section = ($0 == parts || $0 == concerns); next }
  section && match($0, /^- \*\*[^*]+\.\*\*/) {
    line = substr($0, 1, RLENGTH)
    sub(/^- \*\*/, "", line)
    print substr(line, 1, length(line) - 3)
  }
' "${landscape}" | jq -Rsc 'split("\n") | map(select(. != ""))')

# Each line that jq prints starts with a tag that names its refusal.
# shellcheck disable=SC2016 # jq's own variables, not the shell's
found=$(jq -rn \
  --argjson needs "${needs}" \
  --argjson shortlists "${shortlists}" \
  --argjson parts "${parts}" '
  def empty_or_missing: . == null or . == "" or . == [];
  ($needs // {}) as $needs | ($shortlists // {}) as $shortlists
  | ($needs | to_entries[] | .key as $key | .value as $line
    | (if any($parts[]; . == $line.part) then empty
      else "part\t\($key): part \($line.part // "missing")" end),
      (if ($line.words | empty_or_missing) then "words\t\($key)" else empty end),
      (if ($line.stated | empty_or_missing) then "stated\t\($key)" else empty end),
      (if $line.issue == null and $line.declined == null and $shortlists[$key] == null
        then "open\t\($key)" else empty end)),
    ($shortlists | to_entries[] | .key as $need | select($needs[$need] == null)
      | .value | keys[] | "orphan\t\($need).\(.)")
') || cannot '.config/needs.toml and .config/shortlists.toml cannot be judged.'

refuse_tagged() {
  local tag=$1 summary=$2 lines
  lines=$(sed -n "s/^${tag}"$'\t'"//p" <<<"${found}")
  [[ -z "${lines}" ]] || refuse "${summary}" "${lines}"
}

refuse_tagged part \
  "A line of .config/needs.toml is refused when its part is not a part or concern of the \
landscape, because a line states a need of a part of purba. Name the part as the landscape \
names it, or remove the line."

refuse_tagged words \
  "A line of .config/needs.toml is refused when it has no words, because the forecast finds its \
crates on crates.io by each of them. Add a keyword that crates.io holds."

refuse_tagged stated \
  "A line of .config/needs.toml is refused when it has no stated link, because the line that \
states the need holds why purba has it. Link that line."

refuse_tagged open \
  "A line of .config/needs.toml is refused when it has no issue, no declined list and no \
shortlisted crate, because nothing would then vet its crates. Open the issue that vets them, \
and write its link in the issue field."

refuse_tagged orphan \
  "An entry of .config/shortlists.toml is refused when no line of .config/needs.toml names its \
need, because an entry answers its need, and an entry with no need answers nothing. Remove the \
entry, or restore the line."

finish
