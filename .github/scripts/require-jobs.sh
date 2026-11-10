#!/usr/bin/env bash
# Refuse a branch unless every job of a gate did what the job that decides the
# gate asked.
#
#   the caller:  .github/workflows/build.yml
#
#   require-jobs.sh <job> <output>
#
#   NEEDS   the needs context of the gate's last job, as `toJSON(needs)` writes it
#
# <job> decides whether the gate runs its other jobs. When it succeeded and set
# <output> to `true`, each other job must succeed. When it succeeded and set no
# <output>, or an empty one, each other job must skip. A failed or cancelled <job>
# skips every other job too, so a skip passes only when <job> succeeded and asked
# no other job to run.
#
# Exits 0 when every job did what <job> asked, 1 when one did not, and 2 when it
# cannot decide.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/../../scripts/report.sh"

if [[ $# -ne 2 ]]; then
  cannot "usage: require-jobs.sh <job> <output>"
fi
job=$1 output=$2

if [[ -z "${NEEDS:-}" ]]; then
  cannot "NEEDS is unset, so this check reads no job"
fi
if ! jq -e 'type == "object"' >/dev/null 2>&1 <<<"${NEEDS}"; then
  cannot "NEEDS is not a JSON object" "${NEEDS}"
fi
if ! jq -e 'all(.[]; type == "object")' >/dev/null <<<"${NEEDS}"; then
  cannot "NEEDS holds a job that is not a JSON object" "${NEEDS}"
fi
if ! jq -e 'all(.[]; .outputs // {} | type == "object")' >/dev/null <<<"${NEEDS}"; then
  cannot "NEEDS holds a job whose outputs are not a JSON object" "${NEEDS}"
fi
if ! jq -e --arg job "${job}" 'has($job)' >/dev/null <<<"${NEEDS}"; then
  cannot "NEEDS holds no job named \`${job}\`"
fi
if ! jq -e --arg job "${job}" 'del(.[$job]) | length > 0' >/dev/null <<<"${NEEDS}"; then
  cannot "NEEDS holds no job besides ${job}, so this check has nothing to check"
fi

result=$(jq -r --arg job "${job}" '.[$job].result' <<<"${NEEDS}")
value=$(jq -r --arg job "${job}" --arg out "${output}" '.[$job].outputs[$out] // ""' <<<"${NEEDS}")

want="" passed=""
if [[ "${result}" == success && "${value}" == true ]]; then
  want=success passed="${job} set ${output}=true, and every other job succeeded"
elif [[ "${result}" == success && -z "${value}" ]]; then
  want=skipped passed="${job} set no ${output}, and every other job skipped"
fi

if [[ -n "${want}" ]] &&
  jq -e --arg job "${job}" --arg want "${want}" \
    'del(.[$job]) | all(.[]; .result == $want)' >/dev/null <<<"${NEEDS}"; then
  echo "${passed}"
  exit 0
fi

others=$(jq -r --arg job "${job}" 'del(.[$job]) | to_entries[] | "\(.key): \(.value.result)"' \
  <<<"${NEEDS}")
refuse "A branch is refused unless the job \`${job}\` succeeded and every other job did what \
\`${job}\` asked. The gate stands for each of those jobs whenever a path that starts the gate \
changes. \`${job}\` sets ${output}=true to ask for a success, and sets no ${output} to ask for \
a skip." \
  "${job}: ${result}, ${output}=${value}" "${others}"
finish
