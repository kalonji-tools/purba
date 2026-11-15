#!/usr/bin/env bash
# run-github.sh OUT UNTIL
# Runs github.sh once for each GitHub repository that a crate's crates.io `repository` names.
# Needs GH_TOKEN, and OUT/cratesio from fetch-cratesio.sh. Writes OUT/github/repos.tsv
# ("crate<TAB>owner/repo", or "crate<TAB>-" for a repository that is not on GitHub),
# OUT/github/times.tsv, and the API budget used, from `gh api rate_limit` before and after.
set -uo pipefail
out=$1
until=$2
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "${out}/github"
BUDGET='[.resources.core.used, .resources.graphql.used] | @tsv'
for f in "${out}"/cratesio/*.owners.json; do
  c=$(basename "${f}" .owners.json)
  r=$(jq -r '.crate.repository // ""' "${out}/cratesio/${c}.json")
  case ${r} in
    https://github.com/*)
      r=${r#https://github.com/}
      r=${r%%/tree/*}
      r=${r%.git}
      r=${r%/}
      IFS=/ read -r o n _ <<<"${r}"
      printf '%s\t%s/%s\n' "${c}" "${o}" "${n}"
      ;;
    *) printf '%s\t-\n' "${c}" ;;
  esac
done >"${out}/github/repos.tsv"
gh api rate_limit --jq "${BUDGET}" >"${out}/github/budget-before.tsv"
: >"${out}/github/times.tsv"
cut -f2 "${out}/github/repos.tsv" | grep -v '^-$' | sort -u >"${out}/github/repos.list"
while read -r repo; do
  s=$(date +%s)
  bash "${here}/github.sh" "${out}" "${repo}" "${until}" || echo "github.sh failed: ${repo}" >&2
  e=$(date +%s)
  printf '%s\t%s\n' "${repo}" "$((e - s))" >>"${out}/github/times.tsv"
done <"${out}/github/repos.list"
gh api rate_limit --jq "${BUDGET}" >"${out}/github/budget-after.tsv"
