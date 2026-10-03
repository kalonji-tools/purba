#!/usr/bin/env bash
# PROTOTYPE, never merged. Is each candidate maintained?
#
# Reads the latest release and the commits of the last 90 days, never
# `pushed_at`: a push to any branch moves that date.
set -u
since=$(date -u -d '90 days ago' +%Y-%m-%dT%H:%M:%SZ)
printf '| repository | language | licence | latest release | released | commits, 90 days |\n'
printf '|---|---|---|---|---|---:|\n'
for repo in \
  DavidAnson/markdownlint-cli2 \
  igorshubovych/markdownlint-cli \
  rvben/rumdl \
  akiomik/mado \
  ekropotin/quickmark \
  jackdewinter/pymarkdown \
  hukkin/mdformat \
  remarkjs/remark-lint \
  prettier/prettier \
  dprint/dprint-plugin-markdown \
  markdownlint/markdownlint \
  biomejs/biome \
  Automattic/harper \
  crate-ci/typos; do
  meta=$(gh api "repos/${repo}" --jq '[.full_name, (.language // "-"), (.license.spdx_id // "-"), .default_branch, .archived] | @tsv')
  IFS=$'\t' read -r name lang licence branch archived <<<"${meta}"
  # Some projects publish tags and no GitHub release; the newest tag's commit
  # date stands in for those.
  if rel=$(gh api "repos/${name}/releases/latest" --jq '[.tag_name, .published_at[:10]] | @tsv' 2>/dev/null); then
    IFS=$'\t' read -r tag date <<<"${rel}"
  else
    tag=$(gh api "repos/${name}/tags?per_page=1" --jq '.[0].name // "none"')
    sha=$(gh api "repos/${name}/tags?per_page=1" --jq '.[0].commit.sha // empty')
    date=$([[ -n "${sha}" ]] && gh api "repos/${name}/commits/${sha}" --jq '.commit.committer.date[:10]' || printf -- '-')
    tag="${tag} (tag)"
  fi
  commits=$(gh api --paginate "repos/${name}/commits?sha=${branch}&since=${since}&per_page=100" --jq 'length' 2>/dev/null | awk '{s+=$1} END {print s+0}')
  [[ "${archived}" == "true" ]] && name="${name} (ARCHIVED)"
  printf '| %s | %s | %s | %s | %s | %s |\n' "${name}" "${lang}" "${licence}" "${tag}" "${date}" "${commits}"
done
