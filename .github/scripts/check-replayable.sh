#!/usr/bin/env bash
# The replay rule.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#   what you owe:  CONTRIBUTING.md
#
#   check-replayable.sh <base> <head>
#
# <base> is a branch point or a base branch, and this reads the same commits
# either way.
#
# Exits 1 when a commit refuses to replay, and 2 when this script cannot run.
set -euo pipefail

# shellcheck source=.github/scripts/report.sh
. "$(dirname "$0")/report.sh"

if [ $# -ne 2 ]; then
  echo "usage: check-replayable.sh <base> <head>" >&2
  exit 2
fi

if ! base=$(git merge-base "$1" "$2" 2>&1); then
  report "the replay check could not find where $2 leaves $1." "$base"
  exit 2
fi

scratch=$(mktemp -d)
worktree="$scratch/replay"
cleanup() {
  git worktree remove --force "$worktree" 2>/dev/null || true
  rm -rf "$scratch"
}
trap cleanup EXIT

if ! out=$(git worktree add --detach --quiet "$worktree" "$2" 2>&1); then
  report "the replay check could not read the head $2." "$out"
  exit 2
fi

# `git commit` refuses to run without a committer, and a fresh checkout has
# none. This reaches the replay through the environment, so nothing is written
# to the config of the repository this runs in. The author is not set, because
# `--amend --no-edit` keeps the one each commit already carries.
export GIT_COMMITTER_NAME=purba GIT_COMMITTER_EMAIL=purba@invalid

# Keep this command identical to the one `sign.yml` runs, `--exec` included.
export TRAILER="Accepted-by: placeholder <0+placeholder@users.noreply.github.com>"
if replay=$(git -C "$worktree" rebase "$base" --exec \
  'git log -1 --format="%(trailers:key=Accepted-by)" | grep -q . || git commit --amend --no-edit --trailer "$TRAILER"' 2>&1); then
  exit 0
fi

# git names the commit it stopped on, and says which way it stopped. A commit
# that did not apply leaves `stopped-sha` and ends `done` with its own `pick`.
# A commit that replayed empty leaves no `stopped-sha`, because its `pick`
# succeeded and the `exec` after it refused. Every other failure lands there
# too, so the second one is confirmed against the tree rather than assumed.
state="$(git -C "$worktree" rev-parse --absolute-git-dir)/rebase-merge"

detail="the replay left no record of the commit it stopped on."
if stopped=$(grep '^pick ' "$state/done" 2>/dev/null | tail -1 | cut -d' ' -f2) &&
  [ -n "$stopped" ]; then
  detail=$(git log -1 --format='%h %s' "$stopped")
fi

if [ -f "$state/stopped-sha" ]; then
  report "this commit does not apply where purba replays your branch, so the branch cannot be signed. A merge commit whose conflict you resolved by hand is the usual cause. Rebase your branch onto its base instead." "$detail"
elif [ "$(git -C "$worktree" rev-parse 'HEAD^{tree}')" = "$(git -C "$worktree" rev-parse 'HEAD^1^{tree}')" ]; then
  report "this commit replays empty, and purba cannot write its trailer into an empty commit, so the branch cannot be signed. Remove the commit. The replay empties a commit whose change is already on the base as well." "$detail"
else
  report "the replay of this branch stopped here, and this check cannot say why. git wrote what follows." "$detail

$(printf '%s\n' "$replay" | tail -8)"
fi
exit 1
