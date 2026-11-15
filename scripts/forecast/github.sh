#!/usr/bin/env bash
# github.sh OUT OWNER/REPO UNTIL
# Needs a GitHub token. UNTIL is an ISO time such as 2026-11-14T23:59:59Z.
# Writes OUT/github/OWNER_REPO/:
#   review.json     the last 30 commits on the default branch up to UNTIL, each with its pull
#                   requests and their APPROVED reviews (one GraphQL query)
#   review.txt      "<reviewed> <total> <oldest date> <newest date>"; a commit is reviewed when
#                   its first merged pull request has an APPROVED review from a login other than
#                   the pull request's author
#   workflows.json  .github/workflows at the newest of those commits (one GraphQL query)
#   testwf.txt      the workflow files whose text runs the tests, and that run on push,
#                   pull_request or merge_group
#   tests.txt       "observed <url>", or "none"
# A run is observed when a run of a test workflow ends in success, or when one of its jobs ends
# in success and the job's name or a step's name holds "test". The runs at the HEAD come first;
# then the 10 newest runs of each test workflow.
set -euo pipefail
out=$1
repo=$2
until=$3
owner=${repo%/*}
name=${repo#*/}
here=$(cd "$(dirname "$0")" && pwd)
dir=${out}/github/${owner}_${name}
mkdir -p "${dir}"

gh api graphql -F owner="${owner}" -F name="${name}" -F until="${until}" \
  -F query=@"${here}/review.graphql" >"${dir}/review.json"
jq -r '
  .data.repository.defaultBranchRef.target.history.nodes as $n
  | [ $n[]
      | ([.associatedPullRequests.nodes[] | select(.merged)][0]) as $pr
      | if $pr == null then 0
        else ([$pr.reviews.nodes[] | select(.author.login != $pr.author.login)]
              | if length > 0 then 1 else 0 end)
        end ] as $r
  | "\($r | add // 0) \($n | length) \($n[-1].committedDate[0:10]) \($n[0].committedDate[0:10])"
' "${dir}/review.json" >"${dir}/review.txt"

head=$(jq -r '.data.repository.defaultBranchRef.target.history.nodes[0].oid' "${dir}/review.json")
gh api graphql -F owner="${owner}" -F name="${name}" -F expr="${head}:.github/workflows" \
  -F query=@"${here}/workflows.graphql" >"${dir}/workflows.json"
TESTRE='cargo(\s+\+\S+)?\s+(test|nextest|hack\b[^\n]*\btest|miri\s+test|all-features\s+test)'
TESTRE+='|nextest\s+run|\bmake\s+test|\bjust\b[^\n]*\b(test|nextest)|command:\s*test'
TRIGRE='\b(push|pull_request|pull_request_target|merge_group)\b'
jq -r --arg t "${TESTRE}" --arg g "${TRIGRE}" '
  .data.repository.object.entries // [] | .[]
  | select(.name | test("\\.ya?ml$"))
  | select((.object.text // "") | test($t))
  | select((.object.text // "") | test($g))
  | .path' "${dir}/workflows.json" >"${dir}/testwf.txt"

RUNS='.workflow_runs[] | [.id, (.conclusion // "none"), .html_url, (.path | sub("@.*"; ""))] | @tsv'
JOBS='[.jobs[] | select(.conclusion == "success")
        | select((.name | test("test"; "i")) or any(.steps[]?; .name | test("test"; "i")))]
      | .[0].html_url // empty'

observed() { # $1: a runs JSON file; prints the url of the first observed run, or nothing
  jq -r "${RUNS}" "$1" >"$1.tsv"
  while IFS=$'\t' read -r id concl url path; do
    grep -qxF "${path}" "${dir}/testwf.txt" || continue
    if [[ "${concl}" = success ]]; then
      echo "${url}"
      return
    fi
    gh api "repos/${repo}/actions/runs/${id}/jobs?per_page=100" >"${dir}/jobs-${id}.json"
    job=$(jq -r "${JOBS}" "${dir}/jobs-${id}.json")
    if [[ -n "${job}" ]]; then
      echo "${job}"
      return
    fi
  done <"$1.tsv"
}

res=""
if [[ -s "${dir}/testwf.txt" ]]; then
  gh api "repos/${repo}/actions/runs?head_sha=${head}&per_page=100" >"${dir}/runs-head.json"
  res=$(observed "${dir}/runs-head.json")
  if [[ -z "${res}" ]]; then
    while read -r wf; do
      file=$(basename "${wf}")
      gh api "repos/${repo}/actions/workflows/${file}/runs?per_page=10" \
        >"${dir}/runs-${file}.json" || continue
      res=$(observed "${dir}/runs-${file}.json")
      [[ -n "${res}" ]] && break
    done <"${dir}/testwf.txt"
  fi
fi
if [[ -n "${res}" ]]; then echo "observed ${res}"; else echo none; fi >"${dir}/tests.txt"
