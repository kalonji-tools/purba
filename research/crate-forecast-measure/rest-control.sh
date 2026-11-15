#!/usr/bin/env bash
# rest-control.sh OUT OWNER/REPO [all|nopr]
# The control for the review reading of github.sh. For the commits in
# OUT/github/OWNER_REPO/review.json, it applies the #960 audit's rule through the REST API, as
# research/future-crates.md Appendix B3 (measure.sh) does: commits/SHA/pulls (the first with
# merged_at), pulls/N (author), pulls/N/reviews (APPROVED from a login other than the author).
# "nopr" checks only the commits that GraphQL tied to no merged pull request.
# Prints and writes OUT/github/OWNER_REPO/rest-control.tsv:
# "sha graphql-verdict rest-verdict rest-pr". Needs GH_TOKEN.
set -euo pipefail
out=$1
repo=$2
mode=${3:-all}
G=${out}/github/${repo/\//_}
jq -r '.data.repository.defaultBranchRef.target.history.nodes[]
  | ([.associatedPullRequests.nodes[] | select(.merged)][0]) as $pr
  | [ .oid,
      (if $pr == null then "nopr"
        elif ([$pr.reviews.nodes[] | select(.author.login != $pr.author.login)] | length) > 0
        then "reviewed"
        else "unreviewed" end) ]
  | @tsv' "${G}/review.json" >"${G}/graphql-verdicts.tsv"
: >"${G}/rest-control.tsv"
while IFS=$'\t' read -r sha gv; do
  [[ "${mode}" = nopr ]] && [[ "${gv}" != nopr ]] && continue
  pr=$(gh api "repos/${repo}/commits/${sha}/pulls" \
    --jq '[.[] | select(.merged_at != null)][0].number // empty')
  if [[ -z "${pr}" ]]; then
    rv=nopr
  else
    author=$(gh api "repos/${repo}/pulls/${pr}" --jq .user.login)
    gh api --paginate "repos/${repo}/pulls/${pr}/reviews" \
      --jq ".[] | select(.state==\"APPROVED\" and .user.login != \"${author}\") | .user.login" \
      >"${G}/approvals-${pr}.txt"
    if [[ -s "${G}/approvals-${pr}.txt" ]]; then rv=reviewed; else rv=unreviewed; fi
  fi
  printf '%s\t%s\t%s\t%s\n' "${sha}" "${gv}" "${rv}" "${pr:--}" >>"${G}/rest-control.tsv"
done <"${G}/graphql-verdicts.tsv"
cat "${G}/rest-control.tsv"
