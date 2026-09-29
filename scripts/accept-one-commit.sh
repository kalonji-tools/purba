#!/usr/bin/env bash
# The acceptance one commit owes the approval that accepted it.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#   what you owe:  CONTRIBUTING.md
#
#   TRAILER   the `Accepted-by:` line this commit must carry
#
# `git rebase --exec` runs this once per replayed commit, and both the signing
# job and the replay check name this file.
#
# ⚠️ A caller must copy this file out of the repository before it starts the
# rebase, and exec the copy. `--exec` runs against the tree of the commit it
# just replayed, so a path inside the worktree is not there for a commit older
# than this file. `git-rebase(1)` documents none of that, and it was measured:
# the naive wiring refuses the very branch that adds this script.
#
# Exits 2 when it cannot run.
set -euo pipefail

: "${TRAILER:?the acceptance trailer reaches this script through the environment}"

# Read the trailer, never the message. The decision above says why.
#
# Command substitution drops the trailing newline, so this one comparison reads
# all three cases: no trailer is the empty string, one trailer is the whole line
# and equals what is owed, and two hold a newline between them and equal nothing.
carried=$(git log -1 --format='%(trailers:key=Accepted-by)')

# Already exactly the approval that accepted it. Doing nothing here is what
# stops the job repeating forever: the commit does not change, so the branch
# does not, so the caller's push never happens.
if [[ "${carried}" == "${TRAILER}" ]]; then
  exit 0
fi

# Anything else is rewritten, and every `Accepted-by:` is dropped first.
#
# `replace` deletes only the trailer closest to where the new one goes -- see
# `git-interpret-trailers(1)` -- so one pass is not enough and repeating it with
# a value sticks rather than converging. An empty value with `--trim-empty` is
# what makes a pass a deletion instead. Each pass removes one, so the loop ends.
#
# `--parse` reads the trailer block alone. A line in the message body that looks
# like a trailer is therefore never counted and never removed, and git matches a
# key without regard to case, so `accepted-by:` is found as readily.
message=$(git log -1 --format=%B)
while printf '%s\n' "${message}" | git interpret-trailers --parse | grep -qi '^Accepted-by:'; do
  message=$(printf '%s\n' "${message}" |
    git interpret-trailers --if-exists replace --trim-empty --trailer 'Accepted-by:')
done

printf '%s\n' "${message}" |
  git interpret-trailers --trailer "${TRAILER}" |
  git commit --quiet --amend --file -
