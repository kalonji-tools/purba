#!/usr/bin/env bash
# PROTOTYPE, never merged. Does Harper find more in text nobody reviewed?
#
# q3-harper.sh reads merged files, where a reviewer may already have removed
# what Harper would find. This reads drafts instead: the 40 newest issue comments
# by snregales-agent, and what snregales wrote as issue comments, inline review
# comments and review bodies. ⚠️ An agent can post as the human, so the human
# set is not purely the human's.
#
# Prints every lint outside Harper's capitalisation, style and formatting rules,
# for a person to judge.
set -u
cd "$(git rev-parse --show-toplevel)" || exit 2
export MISE_LOCKED=0
H='aqua:Automattic/harper/harper-cli@2.12.0'
d=prototype/out/drafts
rm -rf "${d}" && mkdir -p "${d}/agent" "${d}/human"
repo=repos/kalonji-tools/purba

gh api --paginate "${repo}/issues/comments?per_page=100&sort=created&direction=desc" \
  --jq '.[] | [.user.login, (.id|tostring), (.body|@base64)] | @tsv' | head -400 |
  while IFS=$'\t' read -r who id b64; do
    case "${who}" in snregales-agent) dir=agent ;; snregales) dir=human ;; *) continue ;; esac
    [[ $(find "${d}/${dir}" -type f | wc -l) -ge 40 ]] && continue
    base64 -d <<<"${b64}" >"${d}/${dir}/${id}.md"
  done
gh api --paginate "${repo}/pulls/comments?per_page=100&sort=created&direction=desc" \
  --jq '.[] | select(.user.login == "snregales") | [(.id|tostring), (.body|@base64)] | @tsv' | head -60 |
  while IFS=$'\t' read -r id b64; do base64 -d <<<"${b64}" >"${d}/human/r${id}.md"; done
gh api "${repo}/pulls?state=all&per_page=40" --jq '.[].number' | while read -r n; do
  gh api "${repo}/pulls/${n}/reviews" \
    --jq '.[] | select(.user.login == "snregales" and (.body | length) > 0) | [(.id|tostring), (.body|@base64)] | @tsv'
done | while IFS=$'\t' read -r id b64; do base64 -d <<<"${b64}" >"${d}/human/v${id}.md"; done

for w in agent human; do
  # shellcheck disable=SC2086
  mise x ${H} -- harper-cli lint --format json --quiet --dialect gb "${d}/${w}"/*.md 2>/dev/null |
    jq -c --arg w "${w}" '.[] | .file as $f | .lints[] | {who: $w, file: $f, rule, kind, line, matched_text, s: (.suggestions[0] // "")}'
done >"${d}/harper.jsonl"

printf '| writer | texts | words | lints | spelling | outside capitalisation, style, formatting and spelling |\n'
printf '|---|---:|---:|---:|---:|---:|\n'
for w in agent human; do
  printf '| %s | %s | %s | %s | %s | %s |\n' "${w}" "$(find "${d}/${w}" -type f | wc -l)" \
    "$(cat "${d}/${w}"/*.md | wc -w)" \
    "$(jq -r --arg w "${w}" 'select(.who == $w)' "${d}/harper.jsonl" | jq -s length)" \
    "$(jq -r --arg w "${w}" 'select(.who == $w and .kind == "Spelling")' "${d}/harper.jsonl" | jq -s length)" \
    "$(jq -r --arg w "${w}" 'select(.who == $w and (.kind | test("Capitalization|Style|Formatting|Spelling") | not))' "${d}/harper.jsonl" | jq -s length)"
done

printf '\n### Every lint outside those kinds, with its line\n\n'
jq -r 'select(.kind | test("Capitalization|Style|Formatting|Spelling") | not)
  | [.who, .rule, .matched_text, .s, .file, (.line|tostring)] | @tsv' "${d}/harper.jsonl" |
  while IFS=$'\t' read -r who rule match s file line; do
    printf -- '- %s · %s · `%s` → %s\n  > %s\n' "${who}" "${rule}" "${match}" "${s}" \
      "$(sed -n "${line}p" "${d}/${who}/${file}" | cut -c1-160)"
  done
# The bodies are public comments already; only the lints stay on the branch.
find "${d}" -name '*.md' -delete
