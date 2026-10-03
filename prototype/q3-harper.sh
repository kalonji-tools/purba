#!/usr/bin/env bash
# PROTOTYPE, never merged. What does Harper say about purba's prose, and in
# which dialect?
#
# harper-cli and harper-ls share one engine, so harper-cli's lints are the
# diagnostics an editor would show. Each rule is listed with its three most
# common matched words, so a person can judge the rule in one glance.
set -u
cd "$(git rev-parse --show-toplevel)" || exit 2
export MISE_LOCKED=0
H='aqua:Automattic/harper/harper-cli@2.12.0'
mapfile -t files < <(git ls-files '*.md' ':!:prototype/')
out=prototype/out
mkdir -p "${out}"

for dialect in us gb; do
  # shellcheck disable=SC2086
  mise x ${H} -- harper-cli lint --format json --quiet --dialect "${dialect}" "${files[@]}" \
    2>/dev/null | jq -c '.[] | .file as $f | .lints[] | {file: $f, rule, kind, line, matched_text, suggestions}' \
    >"${out}/harper.${dialect}.jsonl"
done

words=$(cat "${files[@]}" | wc -w)
printf '%s files, %s words\n\n' "${#files[@]}" "${words}"
printf '| dialect | lints | per 1,000 words | files with a lint |\n|---|---:|---:|---:|\n'
for dialect in us gb; do
  n=$(wc -l <"${out}/harper.${dialect}.jsonl")
  nf=$(jq -r .file "${out}/harper.${dialect}.jsonl" | sort -u | grep -c .)
  printf '| %s | %s | %s | %s |\n' "${dialect}" "${n}" \
    "$(awk -v n="${n}" -v w="${words}" 'BEGIN {printf "%.1f", n * 1000 / w}')" "${nf}"
done

printf '\n### Rules, dialect gb\n\n| kind / rule | lints | most matched (count) |\n|---|---:|---|\n'
# shellcheck disable=SC2016 # jq's own syntax, not the shell's
jq -rs 'group_by("\(.kind)/\(.rule)")
  | map({rule: "\(.[0].kind)/\(.[0].rule)", n: length,
      top: (group_by(.matched_text) | map({t: .[0].matched_text, c: length}) | sort_by(-.c) | .[:3])})
  | sort_by(-.n) | .[]
  | "| \(.rule) | \(.n) | \(.top | map("`\(.t)` \(.c)") | join(", ")) |"' "${out}/harper.gb.jsonl"

printf '\n### Lints only the us dialect raises\n\n'
comm -23 <(jq -r '"\(.file):\(.line) \(.matched_text)"' "${out}/harper.us.jsonl" | sort) \
  <(jq -r '"\(.file):\(.line) \(.matched_text)"' "${out}/harper.gb.jsonl" | sort) |
  awk '{$1=""; print}' | sort | uniq -c | sort -rn | head -10
