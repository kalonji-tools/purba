#!/usr/bin/env bash
# The macOS jobs that the workflow of a pull request asks for on one push.
#
#   the decision:  docs/decisions/one-workflow-runs-every-gate-of-a-pull-request.md
#   the task:      mise run lint:macos-jobs
#
#   check-macos-jobs.sh [workflow]
#
# It reads .github/workflows/ci.yml of the repository when it names no workflow.
# A job counts once for each row of its matrix, and a row counts when the
# `runs-on` it gives names macOS. GitHub crosses the lists of a matrix, drops
# each `exclude`, and adds each `include` that matches no row as a row of its own:
# editorconfig-checker-disable-next-line
# https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idstrategymatrixinclude
#
# Exits 1 when it refuses the count, and 2 when it cannot read the workflow.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"

# The Free plan runs 5 macOS jobs at once:
# editorconfig-checker-disable-next-line
# https://docs.github.com/en/actions/reference/limits#job-concurrency-limits-for-github-hosted-runners
cap=5

if [[ $# -gt 0 ]]; then
  workflow=$1
else
  top=$(git rev-parse --show-toplevel 2>/dev/null) ||
    cannot 'with no workflow named, ci.yml can only be found inside a git repository.'
  workflow=${top}/.github/workflows/ci.yml
fi

[[ -r "${workflow}" ]] ||
  cannot "${workflow} cannot be read."
json=$(yq -o json "${workflow}" 2>&1) ||
  cannot "${workflow} cannot be parsed as YAML." "${json}"
jq -e '.jobs | type == "object" and length > 0' >/dev/null 2>&1 <<<"${json}" ||
  cannot "${workflow} names no jobs."

# Prints `<job>: <count>` for each job that asks for macOS.
counts=$(jq -r '
  def rows(m):
    if m == null then [{}] else
    [m | to_entries[] | select(.key != "include" and .key != "exclude")] as $lists
    | [$lists[].key] as $keys
    | (if $lists == [] then []
      else reduce $lists[] as $l ([{}]; [.[] as $r | $l.value[] as $v | $r + {($l.key): $v}])
      end) as $all
    | [$all[] | . as $r | select(any((m.exclude // [])[]; . as $e | $r | contains($e)) | not)]
    | . as $kept
    | [(m.include // [])[] | . as $i
        | select(($kept == [])
            or (any($keys[]; . as $k | $i | has($k))
              and all($kept[]; . as $r
                | any($keys[]; . as $k | ($i | has($k)) and $i[$k] != $r[$k]))))] as $extra
    | ($kept + $extra) | if . == [] then [{}] else . end
    end;
  .jobs | to_entries[]
  | .key as $job | .value as $j
  | [rows($j.strategy.matrix)[] as $r
      | $j."runs-on"
      | if type == "string"
        then gsub("\\$\\{\\{ *matrix\\.(?<k>[A-Za-z0-9_-]+) *\\}\\}"; ($r[.k] // "") | tostring)
        else tojson end
      | select(test("macos"; "i"))]
  | length | select(. > 0)
  | "\($job): \(.)"' <<<"${json}") ||
  cannot "the jobs of ${workflow} cannot be counted."

total=0
while IFS= read -r line; do
  [[ -n "${line}" ]] || continue
  total=$((total + ${line##*: }))
done <<<"${counts}"

if ((total > cap)); then
  refuse "A pull request is refused when its workflow asks for more than ${cap} macOS jobs on \
one push, because the Free plan runs ${cap} at once, and a sixth waits for a runner to come free." \
    "${workflow}: ${total} macOS jobs" "${counts}"
  finish
fi
echo "${workflow}: ${total} macOS jobs, of ${cap}"
