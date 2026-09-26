#!/usr/bin/env bash
# Propose a newer nightly for a person to sign.
#
#   the pin:       mise.toml, under `[tools]`
#   the decision:  docs/decisions/mise-names-every-tool-version.md
#   what you owe:  CONTRIBUTING.md
#
#   bump-nightly.sh <branch> <issue>
#
#   GH_REPO    the repository `gh` acts on
#   GH_TOKEN   a token that may push and open a pull request
#
# Exits 0 when it proposes one, 0 when it deliberately proposes nothing, 1
# when it refuses what it found, and 2 when it cannot run.
set -euo pipefail

if [ $# -ne 2 ]; then
  echo "usage: bump-nightly.sh <branch> <issue>" >&2
  exit 2
fi

branch=$1
issue=$2

# The subject rule requires a reference, so a subject cannot be built without
# one. docs/decisions/a-commit-outlives-its-review.md
if ! [ "$issue" -gt 0 ] 2>/dev/null; then
  echo "bump-nightly.sh needs an issue number, and was given '$issue'." >&2
  exit 2
fi

# The pin lives in these two, and nothing else here may be committed.
toml=mise.toml
lock=mise.lock

# The identity GitHub can attribute to an account. CONTRIBUTING.md says why.
author="github-actions[bot]"
email="41898282+github-actions[bot]@users.noreply.github.com"

if ! git diff --quiet HEAD --; then
  echo "bump-nightly.sh needs a clean tree, because it rewinds to one." >&2
  git --no-pager status --short >&2
  exit 2
fi

# ⚠️ Never replace this with a force-push. A pull request that broke keeps
# the head that refused it, and the logs hanging off that head.
open=$(gh pr list --repo "$GH_REPO" --head "$branch" --state open --json number --jq '.[].number')
if [ -n "$open" ]; then
  echo "pull request #$open is already proposing a nightly on $branch, so this run stands down"
  echo "nothing bumps until a person signs that one or closes it"
  exit 0
fi

read_pin() {
  sed -n 's/^rust = .*version = "\([^"]*\)".*/\1/p' "$toml"
}

before=$(git rev-parse HEAD)
was=$(read_pin)
if [ -z "$was" ]; then
  echo "::error::$toml does not name a rust version this script can read" >&2
  exit 2
fi

mise upgrade --bump rust

# ⚠️ Read the tree, never the exit code.
if git diff --quiet -- "$toml" "$lock"; then
  echo "the pinned nightly is the newest mise offers, so there is nothing to propose"
  exit 0
fi

# ⚠️ Refuse anything this did not ask for. These two files are the ones a
# reviewer of this script has seen.
if ! git diff --quiet -- . ":!$toml" ":!$lock"; then
  echo "::error::the bump changed files beyond $toml and $lock, so it is not proposed"
  git --no-pager diff --stat >&2
  exit 1
fi

now=$(read_pin)
if [ -z "$now" ]; then
  echo "::error::$toml no longer names a rust version this script can read" >&2
  exit 2
fi

echo "proposing $now, which replaces $was"

subject="chore: move the nightly to ${now#nightly-} (#$issue)"

# ⚠️ No `-s`. CONTRIBUTING.md: a machine never writes that trailer.
git -c "user.name=$author" -c "user.email=$email" \
  commit --quiet -m "$subject" -- "$toml" "$lock"

# The branch outlives a closed pull request, so this replaces it.
git push --force origin "HEAD:refs/heads/$branch"

# Leave the caller the tree it checked out.
git reset --quiet --hard "$before"

body=$(cat <<BODY
A machine wrote this. It proposes a compiler and certifies nothing.

| | |
|---|---|
| from | \`$was\` |
| to | \`$now\` |
| files | \`$toml\` and \`$lock\`, one line each |

⚠️ **\`Origin\` is red on this branch, and that is correct.** A machine never writes a \`Signed-off-by:\` trailer, so a person adds it to these commits:

\`\`\`
git fetch origin $branch
git switch --detach FETCH_HEAD
mise run sign-off
git push --force-with-lease origin HEAD:$branch
\`\`\`

**If \`Build\` is red, this nightly broke purba.** Leave this pull request open and nothing bumps until somebody closes it: the weekly run stands down while it is here, so the compiler that broke keeps the head that refused it.

Opened by \`.github/workflows/bump.yml\`, which [#$issue](https://github.com/$GH_REPO/issues/$issue) owns.
BODY
)

gh pr create --repo "$GH_REPO" --base main --head "$branch" \
  --title "$subject" \
  --body "$body"
