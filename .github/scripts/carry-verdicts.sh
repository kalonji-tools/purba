#!/usr/bin/env bash
# Carry a verdict across a rewrite that changed no content.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#   what you owe:  CONTRIBUTING.md
#
#   carry-verdicts.sh <before> <after> <context>...
#
# Never name a context here that reads a commit message. The rewrite edits the
# thing it reads, so its answer is genuinely different and carrying it lies.
#
# Exits 0 whether or not anything was carried, and 2 when it cannot run. The
# caller must not let a failure here fail the signing job: a branch that is
# signed and uncarried is recovered by restarting the run, and one that is
# unsigned is not.
set -euo pipefail

if [[ $# -lt 3 ]]; then
  echo "usage: carry-verdicts.sh <before> <after> <context>..." >&2
  exit 2
fi

# shellcheck source=.github/scripts/check-run.sh
. "$(dirname "$0")/check-run.sh"

before=$1
after=$2
shift 2

# Nothing moved, so the verdicts are already where they belong. Without this
# the loop below would read a commit's own verdict and post it back onto it.
if [[ "${before}" = "${after}" ]]; then
  echo "the head did not move, so there is nothing to carry"
  exit 0
fi

if ! before_tree=$(git rev-parse --verify --quiet "${before}^{tree}") ||
  ! after_tree=$(git rev-parse --verify --quiet "${after}^{tree}"); then
  echo "the tree of ${before} or of ${after} could not be read." >&2
  exit 2
fi

echo "before ${before} tree ${before_tree}"
echo "after  ${after} tree ${after_tree}"

if [[ "${before_tree}" != "${after_tree}" ]]; then
  echo "the content changed, so no verdict is carried"
  exit 0
fi

for context in "$@"; do
  runs=$(check_runs_of "${before}")
  conclusion=$(last_conclusion "${context}" <<<"${runs}")

  # Absent reads the same as refused here, on purpose.
  case "${conclusion}" in
    # Never widen this list.
    success | failure) ;;
    "")
      echo "${context} has no verdict on ${before}, so there is nothing to carry"
      continue
      ;;
    *)
      echo "${context} concluded ${conclusion} on ${before}, which is not a verdict, so there is \
nothing to carry"
      continue
      ;;
  esac

  # The conclusion goes across as it stands. A refusal stays a refusal.
  carried_summary="The tree at this commit is \`${after_tree}\`, which is the tree this \
check answered for at \`${before}\`."
  post_check_run "${context}" "${after}" "${conclusion}" "${context}, carried" \
    "${carried_summary}"

  echo "carried ${context}=${conclusion}"
done
