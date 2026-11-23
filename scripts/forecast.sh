#!/usr/bin/env bash
# The crates that could meet each stated need of a part not built yet, each measured by the
# levels of the crate record, and the body of one issue for each need, which vets them.
#
#   the decision:  docs/decisions/a-forecast-finds-the-crates-for-each-stated-need.md
#   the task:      mise run list:needs
#
#   forecast.sh                 each line with no issue, no declined list and no shortlisted crate
#   forecast.sh --part <part>   those lines of <part>, and each of its declined lines
#   forecast.sh --shortlisted   each entry of .config/shortlists.toml again, at its version,
#                               with the versions and commits before its measured day. Prints
#                               each level, unmeasured and needs that moved
#
# It asks crates.io at most once a second, and reads GitHub with the token of gh. The measure
# builds each crate, so it runs the build scripts and proc macros of the crate and of what it
# brings. It measures in a scratch directory, and removes it.
#
# It writes neither .config/needs.toml nor .config/shortlists.toml.
#
# Exits 2 when it cannot run.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"
here=$(cd "$(dirname "$0")" && pwd) ||
  cannot "the directory of $0 cannot be found."

part=""
shortlisted=no
while [[ $# -gt 0 ]]; do
  case $1 in
    --part)
      [[ $# -ge 2 && -n "$2" ]] || cannot '--part names a part or concern of the landscape.'
      part=$2
      shift 2
      ;;
    --shortlisted)
      shortlisted=yes
      shift
      ;;
    *) cannot "forecast.sh takes --part <part> or --shortlisted, and not $1." ;;
  esac
done

top=$(git rev-parse --show-toplevel 2>/dev/null) ||
  cannot 'the needs can only be found inside a git repository.'
cd "${top}"

needs=$(yq -p toml -o json -I0 . .config/needs.toml) ||
  cannot '.config/needs.toml cannot be read.'
shortlists=$(yq -p toml -o json -I0 . .config/shortlists.toml) ||
  cannot '.config/shortlists.toml cannot be read.'
needs=$(jq -c '. // {}' <<<"${needs}")
shortlists=$(jq -c '. // {}' <<<"${shortlists}")

work=$(mktemp -d) || cannot 'the scratch directory of the measure cannot be made.'
trap 'rm -rf -- "${work}"' EXIT
mkdir -p "${work}/cratesio"
: >"${work}/units.tsv"
: >"${work}/measured.jsonl"

agent='purba (https://github.com/kalonji-tools/purba)'

# ask <url>: crates.io's answer. crates.io asks for at most one request a second.
ask() {
  curl -sS -f -A "${agent}" "$1" || cannot "crates.io cannot be asked for $1."
  sleep 1.1
}

# Each crate that an entry of .config/shortlists.toml holds, with the part of its need.
# shellcheck disable=SC2016 # jq's own variables, not the shell's
held=$(jq -c --argjson needs "${needs}" '
  [to_entries[] | ($needs[.key].part // empty) as $part | .value | keys[] | [., $part]]
' <<<"${shortlists}")

if [[ "${shortlisted}" == yes ]]; then
  # shellcheck disable=SC2016 # jq's own variables, not the shell's
  jq -r --argjson held "${held}" '
    to_entries[] | .value | to_entries[]
    | .key as $crate
    | ([$held[] | select(.[0] == $crate)] | unique | length) as $parts
    | [$crate, .value.version, $parts, "\(.value.measured)T00:00:00Z"] | @tsv
  ' <<<"${shortlists}" | sort -u >"${work}/units.tsv"
  lines='[]'
else
  # shellcheck disable=SC2016 # jq's own variables, not the shell's
  lines=$(jq -c --arg part "${part}" --argjson shortlists "${shortlists}" '
    [to_entries[] | select(.value.issue == null and $shortlists[.key] == null)
      | select(if $part == "" then .value.declined == null else .value.part == $part end)]
  ' <<<"${needs}")

  # Each line, with the crates its words find that it has not declined, in the order found.
  wanted=$(jq -r '.[] | [.key, (.value.words | join(" ")), (.value.declined // null | tojson)]
    | @tsv' <<<"${lines}")
  found='[]'
  while IFS=$'\t' read -r key words declined; do
    [[ -n "${key}" ]] || continue
    crates='[]'
    read -ra each <<<"${words}"
    for word in "${each[@]}"; do
      query="keyword=${word}&sort=recent-downloads&per_page=20"
      answer=$(ask "https://crates.io/api/v1/crates?${query}")
      names=$(jq -c '[.crates[].name]' <<<"${answer}")
      [[ "${names}" != '[]' ]] || printf '%s: the word %s finds no crate\n' "${key}" "${word}" >&2
      # shellcheck disable=SC2016 # jq's own variables, not the shell's
      crates=$(jq -c --argjson names "${names}" --argjson declined "${declined}" '
        reduce ($names[] | select(. as $name | $declined | index($name) | not)) as $name
          (.; if index($name) then . else . + [$name] end)' <<<"${crates}")
    done
    if [[ "${crates}" == '[]' ]]; then
      if [[ "${declined}" == null ]]; then
        printf '%s: no crate; write declined = []\n' "${key}" >&2
      else
        printf '%s: no crate beyond declined = %s\n' "${key}" "${declined}" >&2
      fi
      continue
    fi
    # shellcheck disable=SC2016 # jq's own variables, not the shell's
    found=$(jq -c --arg key "${key}" --argjson crates "${crates}" \
      '. + [{key: $key, crates: $crates}]' <<<"${found}")
  done <<<"${wanted}"

  # Each crate found, with the part of each line that holds it.
  # shellcheck disable=SC2016 # jq's own variables, not the shell's
  pairs=$(jq -c --argjson held "${held}" --argjson needs "${needs}" '
    $held + [.[] | $needs[.key].part as $part | .crates[] | [., $part]] | unique' <<<"${found}")
  listed=$(jq -r '[.[].crates[]] | unique[]' <<<"${found}")
  now=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  while read -r crate; do
    [[ -n "${crate}" ]] || continue
    ask "https://crates.io/api/v1/crates/${crate}" >"${work}/cratesio/${crate}.json"
    # shellcheck disable=SC2016 # jq's own variables, not the shell's
    version=$(jq -r --arg until "${now}" '
      [.versions[] | select(.yanked | not) | select(.num | split("+")[0] | test("-") | not)
        | select(.created_at < $until)] | max_by(.created_at) | .num // empty
    ' "${work}/cratesio/${crate}.json")
    if [[ -z "${version}" ]]; then
      printf '%s: no version that is stable and not yanked\n' "${crate}" >&2
      continue
    fi
    count=$(jq -r --arg crate "${crate}" '[.[] | select(.[0] == $crate)] | length' <<<"${pairs}")
    printf '%s\t%s\t%s\t%s\n' "${crate}" "${version}" "${count}" "${now}" >>"${work}/units.tsv"
  done <<<"${listed}"
fi

[[ -s "${work}/units.tsv" ]] || exit 0
gh auth token >/dev/null 2>&1 ||
  cannot 'the measure reads GitHub, and gh holds no token. Run gh auth login.'

deny=()
[[ ! -e .cargo/deny.toml ]] || deny=(--config .cargo/deny.toml)

# check <probe> <check>: each error of that cargo-deny check on a crate of the probe, or pass.
# A licence check names how many crates fail for each finding, because most of a graph can fail it.
check() {
  local written read
  [[ -s "$1/metadata.json" ]] || {
    echo 'not run'
    return
  }
  written=$(cargo-deny --format json "${deny[@]}" --metadata-path "$1/metadata.json" \
    check "$2" 2>&1 >/dev/null) || true
  read=$(jq -Rn '[inputs | fromjson? // empty]' <<<"${written}")
  if ! jq -e 'any(.[]; .type == "summary")' <<<"${read}" >/dev/null; then
    echo 'not run'
    return
  fi
  # A finding that names no crate is about the configuration, not about the crate.
  # shellcheck disable=SC2016 # jq's own variables, not the shell's
  jq -r --arg check "$2" '[.[] | select(.type == "diagnostic" and .fields.severity == "error")
    | .fields | (.graphs[0].Krate // null) as $crate
    | select($crate != null and ($crate.name | startswith("probe_") | not))
    | {crate: "\($crate.name) \($crate.version)", finding: (.advisory.id // .code)}] | unique
    | if . == [] then "pass"
      elif $check == "licenses" then
        group_by(.finding) | map("\(length) \(.[0].finding)") | join(", ")
      else map("\(.crate): \(.finding)") | join("; ") end' <<<"${read}"
}

while IFS=$'\t' read -r crate version count until; do
  out=${work}/cut-${until//:/}
  mkdir -p "${out}/github"
  [[ -e "${out}/cratesio" ]] || ln -s ../cratesio "${out}/cratesio"
  if [[ ! -e "${work}/cratesio/${crate}.json" ]]; then
    ask "https://crates.io/api/v1/crates/${crate}" >"${work}/cratesio/${crate}.json"
  fi
  if [[ ! -e "${work}/cratesio/${crate}.owners.json" ]]; then
    ask "https://crates.io/api/v1/crates/${crate}/owners" >"${work}/cratesio/${crate}.owners.json"
  fi

  repo=$(jq -r '.crate.repository // ""' "${work}/cratesio/${crate}.json")
  case ${repo} in
    https://github.com/*)
      repo=${repo#https://github.com/}
      repo=${repo%%/tree/*}
      repo=${repo%.git}
      repo=${repo%/}
      IFS=/ read -r owner name _ <<<"${repo}"
      repo=${owner}/${name}
      ;;
    *) repo=- ;;
  esac
  printf '%s\t%s\n' "${crate}" "${repo}" >>"${out}/github/repos.tsv"
  if [[ "${repo}" != - && ! -e "${out}/github/${owner}_${name}/tests.txt" ]]; then
    bash "${here}/forecast/github.sh" "${out}" "${repo}" "${until}" ||
      printf '%s: GitHub cannot be read for %s\n' "${crate}" "${repo}" >&2
  fi

  bash "${here}/forecast/probe.sh" "${out}" "${crate}" "${version}" ||
    printf '%s %s: the probe cannot be built\n' "${crate}" "${version}" >&2
  printf '%s\t%s\t%s\n' "${crate}" "${version}" "${count}" >"${out}/crates.tsv"
  row=$(bash "${here}/forecast/levels.sh" "${out}" "${until}" "${top}") ||
    cannot "the levels of ${crate} ${version} cannot be read."

  checks='{}'
  if [[ "${shortlisted}" == no ]]; then
    advisories=$(check "${out}/probe/${crate}" advisories)
    licences=$(check "${out}/probe/${crate}" licenses)
    sources=$(check "${out}/probe/${crate}" sources)
    # shellcheck disable=SC2016 # jq's own variables, not the shell's
    checks=$(jq -nc --arg advisories "${advisories}" --arg licences "${licences}" \
      --arg sources "${sources}" '{$advisories, $licences, $sources}')
  fi
  # shellcheck disable=SC2016 # jq's own variables, not the shell's
  jq -Rc -L "${here}" --arg until "${until}" --argjson checks "${checks}" '
    include "crate-pain";
    split("\t") as $row
    | ([levels] | to_entries | map({key: .value, value: ($row[.key + 2] | tonumber? // null)})
      | from_entries) as $measured
    | {crate: $row[0], version: $row[1], $until, $checks,
      levels: ($measured | map_values(. // 3)),
      unmeasured: [levels as $level | select($measured[$level] == null) | $level]}
  ' <<<"${row}" >>"${work}/measured.jsonl"
done <"${work}/units.tsv"

# shellcheck disable=SC2016 # jq's own variables, not the shell's
jq -rn -L "${here}" --argjson needs "${needs}" --argjson shortlists "${shortlists}" \
  --arg shortlisted "${shortlisted}" --argjson found "${found:-[]}" '
  include "crate-pain";
  [inputs] as $measured
  | def measure($crate; $version; $until):
      first($measured[] | select(.crate == $crate and .version == $version
        and (.until | startswith($until))));
    def row:
      fourteenths(.levels) as $sum
      | [("`" + .crate + "`"), .version, .checks.advisories, .checks.licences,
        .checks.sources, (.levels[levels] | tostring),
        (if .unmeasured == [] then "—" else .unmeasured | join(", ") end),
        hundredths($sum), (needs($sum) | tostring)]
      | "| " + join(" | ") + " |";

  if $shortlisted == "yes" then
    $shortlists | to_entries[] | .key as $need | .value | to_entries[]
    | .key as $crate | .value as $entry
    | measure($crate; $entry.version; $entry.measured) as $now
    | [(levels as $level | select($entry[$level] != $now.levels[$level])
        | "\($level) \($entry[$level])→\($now.levels[$level])"),
      (select(($entry.unmeasured // []) != $now.unmeasured)
        | "unmeasured \($entry.unmeasured // [] | tojson)→\($now.unmeasured | tojson)"),
      (needs(fourteenths($now.levels)) as $level | select($entry.needs != $level)
        | "needs \($entry.needs)→\($level)")]
    | select(. != [])
    | "\($need).\($crate) \($entry.version): \(join(", "))"
  else
    $found[] | .key as $key | $needs[$key] as $line
    | [.crates[] as $crate | first($measured[] | select(.crate == $crate))] as $rows
    | select($rows != [])
    | "==> \($key) <==\n",
      "- need: \($line.need)",
      "- part: \($line.part)",
      "- words: \($line.words | map("`" + . + "`") | join(", "))",
      (select($line.have) | "- has: \($line.have)"),
      "- stated: \($line.stated)",
      "",
      "Measured with the cut at \($rows[0].until), by `mise run list:needs`.",
      "",
      "| crate | version | advisories | licences | sources | blast area | upkeep | maintainers" +
        " | upstream review | tests | `unsafe` | API churn | owner in purba | unmeasured" +
        " | pain | needs |",
      "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|",
      ($rows[] | row),
      ""
  end
' "${work}/measured.jsonl"
