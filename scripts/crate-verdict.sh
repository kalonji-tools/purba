#!/usr/bin/env bash
# Whether each crate that a manifest names joins purba, calculated from the
# levels in .config/crates.toml and the findings of cargo-deny, and whether each
# entry of .config/shortlists.toml holds the needs that its levels give.
#
#   the decision:  docs/decisions/a-crate-joins-when-its-relief-exceeds-its-pain.md
#   the task:      mise run lint:crates
#
#   crate-verdict.sh
#
# A graph is the workspace, or one cargo script with the Cargo.lock beside it.
# cargo-deny runs the advisory and the source checks on each graph. For a cargo
# script in scripts/<name>/, it reads .cargo/<name>.deny.toml when that exists.
# It fetches the advisory database, so the check needs the network.
#
# Exits 1 when it refuses, and 2 when a graph or the levels cannot be read.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"
here=$(cd "$(dirname "$0")" && pwd) ||
  cannot "the directory of $0 cannot be found."

top=$(git rev-parse --show-toplevel 2>/dev/null) ||
  cannot 'the manifests can only be found inside a git repository.'
cd "${top}"

scores=$(yq -p toml -o json -I0 . .config/crates.toml) ||
  cannot '.config/crates.toml cannot be read.'
shortlists=$(yq -p toml -o json -I0 . .config/shortlists.toml) ||
  cannot '.config/shortlists.toml cannot be read.'
records=$(git ls-files -z 'docs/decisions/*.md' |
  jq -Rsc 'split("\u0000") | map(select(. != ""))') ||
  cannot 'the tracked records cannot be listed.'
scripts_list=$(git ls-files 'scripts/*.rs' '.github/*.rs') ||
  cannot 'the tracked cargo scripts cannot be listed.'

work=$(mktemp -d)
trap 'rm -rf -- "${work}"' EXIT
named='[]'
findings='[]'
graphs=0

# judge <graph> <config of cargo-deny, or nothing> <argument of cargo metadata>...
judge() {
  local graph=$1 file=$2 metadata written read logged
  local -a config=()
  shift 2
  graphs=$((graphs + 1))
  metadata="${work}/metadata-${graphs}.json"
  cargo metadata "$@" >"${metadata}" ||
    cannot "the crates of ${graph} cannot be read."

  # cargo-deny writes its findings to standard error, and exits 1 when it finds one.
  [[ ! -e "${file}" ]] || config=(--config "${file}")
  written=$(cargo-deny --format json "${config[@]}" --metadata-path "${metadata}" \
    check advisories sources 2>&1 >/dev/null) || true
  read=$(jq -Rn '[inputs | fromjson? // empty]' <<<"${written}")
  if ! jq -e 'any(.[]; .type == "summary")' <<<"${read}" >/dev/null; then
    logged=$(jq -r 'map(select(.type == "log") | .fields.message) | join("\n")' <<<"${read}")
    cannot "cargo-deny cannot check ${graph}." "${logged}"
  fi

  # shellcheck disable=SC2016 # jq's own variables, not the shell's
  named=$(jq -c --arg graph "${graph}" --argjson named "${named}" '
    (.packages | map({key: .id, value: .}) | from_entries) as $package
    | $named + [
      .workspace_members[] as $member
      | [$package[$member].dependencies[] | select(.source != null) | .name] as $names
      | .resolve.nodes[] | select(.id == $member) | .deps[]
      | $package[.pkg] | select(.name as $name | any($names[]; . == $name))
      | {graph: $graph, crate: .name, version}
    ] | unique' "${metadata}")

  # shellcheck disable=SC2016 # jq's own variables, not the shell's
  findings=$(jq -c --arg graph "${graph}" --argjson findings "${findings}" '
    $findings + [
      .[] | select(.type == "diagnostic" and .fields.severity == "error") | .fields
      | (.graphs[0].Krate // {}) as $crate
      | {graph: $graph, crate: $crate.name, version: $crate.version,
        id: (.advisory.id // (if $crate.name then "source" else .code end))}
    ]' <<<"${read}")
}

judge 'the workspace' '' --format-version 1 --locked
while read -r script; do
  [[ -n "${script}" ]] || continue
  directory=${script%/*}
  judge "${script}" ".cargo/${directory##*/}.deny.toml" -Zscript --format-version 1 --locked \
    --manifest-path "${script}" --config "resolver.lockfile-path=\"${PWD}/${directory}/Cargo.lock\""
done <<<"${scripts_list}"

# Each line it prints starts with the tag of the refusal it belongs to.
# shellcheck disable=SC2016 # jq's own variables, not the shell's
verdicts=$(jq -nr -L "${here}" \
  --argjson scores "${scores}" \
  --argjson shortlists "${shortlists}" \
  --argjson named "${named}" \
  --argjson findings "${findings}" \
  --argjson records "${records}" '
  include "crate-pain";

  def factors: levels, "relief";
  def level($l): any(1, 2, 3; . == $l);
  def failure: tostring | test("^(RUSTSEC-[0-9]{4}-[0-9]{4}|source|score)$");
  def comment:
    test("^https:/{2}github[.]com/kalonji-tools/purba/issues/[0-9]+#issuecomment-[0-9]+$");
  def among($list): . as $item | any($list[]; . == $item);

  [$named[] | "\(.crate)@\(.version)"] as $locked
  | ($findings[] | select(.crate == null or ("\(.crate)@\(.version)" | among($locked) | not))
    | if .crate then "gate\t\(.crate) \(.version), in \(.graph): \(.id)"
      else "gate\t\(.graph): \(.id)" end),

  ($named | group_by(.crate)[] | . as $locks | .[0].crate as $crate | $scores[$crate] as $entry
    | if $entry == null then ($locks[] | "missing\t\($crate) \(.version), in \(.graph)")
      else
        ($locks[] | select(.version != $entry.version)
          | "version\t\($crate): the entry judges \($entry.version)," +
            " and \(.graph) locks \(.version)"),

        ($entry.scored | if . == null then "scored\t\($crate): no scored"
          elif comment or (startswith("docs/decisions/") and among($records)) then empty
          else "scored\t\($crate): \(.)" end),

        (factors as $factor | $entry[$factor] as $level
          | if $level == null then "levels\t\($crate): \($factor) has no level"
            elif level($level) then empty
            else "levels\t\($crate): \($factor) is \($level)" end),

        (select(all(factors; level($entry[.])))
          | fourteenths($entry) as $sum
          | [$locks[].version] as $versions
          | ([$findings[] | select(.crate == $crate and (.version | among($versions))) | .id]
            | unique) + (if joins($entry.relief; $sum) then [] else ["score"] end)
          | . as $fails
          | $entry.waiver as $waiver
          | if $waiver == null then
              if $fails == [] then empty
              else "out\t\($crate): pain \(hundredths($sum)), relief \($entry.relief)," +
                " fails \($fails | join(", "))" end
            else
              ($waiver.fails // []) as $waived
              | (if $waived == [] then "malformed\t\($crate): no failure" else empty end),
                ($waived[] | select(failure | not) | "malformed\t\($crate): \(.)"),
                (if ($waiver.why // "") == "" then "malformed\t\($crate): no why" else empty end),
                (if ($waiver.issue // "") == "" then "malformed\t\($crate): no issue"
                  else empty end),
                ($waived[] | select(failure and (among($fails) | not)) | "excess\t\($crate): \(.)"),
                ($fails[] | select(among($waived) | not) | "unwaived\t\($crate): \(.)")
            end)
      end),

  ($shortlists // {} | to_entries[] | .key as $need | .value | to_entries[]
    | "\($need).\(.key)" as $name | .value as $entry
    | (levels as $factor | $entry[$factor] as $level
        | if $level == null then "shortlisted-levels\t\($name): \($factor) has no level"
          elif level($level) then empty
          else "shortlisted-levels\t\($name): \($factor) is \($level)" end),

      (select(all(levels; level($entry[.])))
        | fourteenths($entry) as $sum | needs($sum) as $needs
        | select($entry.needs != $needs)
        | "shortlisted-needs\t\($name): needs \($entry.needs), and its levels need \($needs)" +
          " (pain \(hundredths($sum)))"))
')

tagged() {
  sed -n "s/^$1"$'\t'"//p" <<<"${verdicts}"
}

refuse_tagged() {
  local found
  found=$(tagged "$1")
  [[ -z "${found}" ]] || refuse "$2" "${found}"
}

refuse_tagged gate \
  "A finding of the advisory or the source check is refused when it names no crate that a \
manifest names, because no waiver can let it through. Fix the crate, or ignore the advisory \
in the deny.toml of its graph with a reason."

refuse_tagged missing \
  "A crate that a manifest names is refused when .config/crates.toml holds no entry for it, \
because its verdict is calculated from that entry. Add an entry with a level for each factor."

refuse_tagged version \
  "An entry is refused when its version differs from the version the lock holds, because its \
levels judge one version. Judge the whole entry again for the locked version."

refuse_tagged levels \
  "An entry is refused when a factor has no level from 1 to 3, because its pain and its \
verdict are calculated from each level."

refuse_tagged scored \
  "An entry is refused when scored names neither a record under docs/decisions/ nor an issue \
comment, because scored links where a person scored the relief and the trust."

refuse_tagged malformed \
  "A waiver is refused when it names anything except an advisory id, source or score, or has \
no why or no issue, because a person decides each failure it lets through, and says why and \
where."

refuse_tagged excess \
  "A waiver is refused when it names a failure its crate does not have, because a waiver that \
lets nothing through outlives its reason. Remove that failure from the waiver."

refuse_tagged unwaived \
  "A crate is refused when it fails something its waiver does not name, because a waiver lets \
through only the failures a person decided."

refuse_tagged out \
  "A crate is refused when a check fails or its relief does not exceed its pain, and no waiver \
lets it in, because a crate joins only when each check passes and its relief exceeds its pain."

refuse_tagged shortlisted-levels \
  "An entry of .config/shortlists.toml is refused when a level is not 1, 2 or 3, because \
lint:crates calculates its pain and its needs from each level."

refuse_tagged shortlisted-needs \
  "An entry of .config/shortlists.toml is refused when its needs value differs from the lowest \
relief that lets its crate join at the pain its levels give, because a change to a level or a \
weight must not move that relief unseen."

finish
