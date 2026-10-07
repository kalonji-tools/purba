#!/usr/bin/env bash
# The replay rule.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#   what you owe:  CONTRIBUTING.md
#
#   check-replayable.sh <base> <head>
#
#   TRAILER   the `Accepted-by:` line the replay writes, and a placeholder when unset
#
# <base> is a branch point or a base branch, and this reads the same commits
# either way.
#
# Prints the replayed head when the replay keeps the content. Exits 1 when a
# commit refuses to replay, or when the replay completes and loses content.
# Exits 2 when this script cannot run.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"

[[ $# -eq 2 ]] || cannot "usage: check-replayable.sh <base> <head>"

base=$(git merge-base "$1" "$2" 2>&1) ||
  cannot "the replay check could not find where $2 leaves $1." "${base}"

# Inside the worktree below, the name `HEAD` is the replayed commit, not this one.
head=$(git rev-parse "$2^{commit}")

scratch=$(mktemp -d)
worktree="${scratch}/replay"
# `trap cleanup EXIT` below is the caller, which shellcheck cannot see.
# shellcheck disable=SC2329
cleanup() {
  git worktree remove --force "${worktree}" 2>/dev/null || true
  rm -rf "${scratch}"
}
trap cleanup EXIT

out=$(git worktree add --detach --quiet "${worktree}" "${head}" 2>&1) ||
  cannot "the replay check could not make a worktree to replay $2 in." "${out}"

# `git commit` refuses to run without a committer, and a fresh checkout has
# none. This reaches the replay through the environment, so nothing is written
# to the config of the repository this runs in. The author is not set, because
# `--amend` keeps the one each commit already carries.
export GIT_COMMITTER_NAME="${GIT_COMMITTER_NAME:-purba}"
export GIT_COMMITTER_EMAIL="${GIT_COMMITTER_EMAIL:-purba@invalid}"

placeholder="Accepted-by: placeholder <0+placeholder@users.noreply.github.com>"
export TRAILER="${TRAILER:-${placeholder}}"

# `accept-one-commit.sh` says why the copy below is load-bearing.
#
# With no `TRAILER` given, the placeholder above matches nothing a commit
# carries, so every commit is rewritten in the replay. That costs this check
# nothing: it compares trees, and a trailer lives in the message.
accept_exec="${scratch}/accept-one-commit.sh"
cp "$(dirname "$0")/accept-one-commit.sh" "${accept_exec}"
if replay=$(git -C "${worktree}" rebase "${base}" --exec "${accept_exec}" 2>&1); then
  replayed_tree=$(git -C "${worktree}" rev-parse 'HEAD^{tree}') || exit 2
  head_tree=$(git rev-parse "${head}^{tree}") || exit 2
  if [[ "${replayed_tree}" = "${head_tree}" ]]; then
    git -C "${worktree}" rev-parse HEAD || exit 2
    exit 0
  fi

  # The cause is read rather than assumed.
  changed=$(git -C "${worktree}" diff --name-status "${head}" HEAD)
  merges=$(git log --merges --format='%h %s' "${base}..${head}")

  if [[ -n "${merges}" ]]; then
    refuse "A branch is refused when its replay changes its content, because the rewrite that \
signs it must leave your content untouched. The replay flattens a merge commit, and the changes \
made in that merge are lost. Rebase your branch onto its base instead." "${changed}

${merges}"
  else
    refuse "A branch is refused when its replay changes its content, because the rewrite that \
signs it must leave your content untouched. This check cannot say why the content changed, \
because the branch carries no merge commit." "${changed}"
  fi
  finish
fi

# git names the commit it stopped on, and says which way it stopped. A commit
# that did not apply leaves `stopped-sha` and ends `done` with its own `pick`.
# A commit that replayed empty leaves no `stopped-sha`, because its `pick`
# succeeded and the `exec` after it refused. Every other failure lands there
# too, so the second one is confirmed against the tree rather than assumed.
state="$(git -C "${worktree}" rev-parse --absolute-git-dir)/rebase-merge"

detail="the replay left no record of the commit it stopped on."
if stopped=$(grep '^pick ' "${state}/done" 2>/dev/null | tail -1 | cut -d' ' -f2) &&
  [[ -n "${stopped}" ]]; then
  detail=$(git log -1 --format='%h %s' "${stopped}")
fi

# `HEAD^1` has no parent to resolve on a root commit, so its absence is read as a
# value rather than as a failure. The tree of the replayed commit is a failure:
# the replay reached it.
stopped_tree=$(git -C "${worktree}" rev-parse 'HEAD^{tree}') || exit 2
parent_tree=$(git -C "${worktree}" rev-parse 'HEAD^1^{tree}' 2>/dev/null || true)
replay_tail=$(printf '%s\n' "${replay}" | tail -8)

if [[ -f "${state}/stopped-sha" ]]; then
  refuse "A commit is refused when it does not apply where purba replays your branch, because \
purba writes its Accepted-by trailer into each commit it replays. A merge commit whose conflict \
you resolved by hand is the usual cause. Rebase your branch onto its base instead." "${detail}"
elif [[ -n "${stopped}" && -n "${parent_tree}" && "${stopped_tree}" = "${parent_tree}" ]]; then
  refuse "A commit is refused when it replays empty, because purba writes its Accepted-by trailer \
into each commit it replays and an empty commit cannot hold one. Remove the commit. The replay \
empties a commit whose change is already on the base as well." "${detail}"
else
  refuse "A branch is refused when its replay stops, because purba writes its Accepted-by trailer \
into each commit it replays. This check cannot say why it stopped here. git wrote what follows." \
    "${detail}

${replay_tail}"
fi
finish
