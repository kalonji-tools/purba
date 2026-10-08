#!/usr/bin/env bash
# Tag the version `main` names, or propose the next one for a person to sign.
#
#   the register:  .config/cliff.toml
#   what you owe:  CONTRIBUTING.md
#
#   release.sh <branch> <issue>
#
#   GH_REPO    the repository `gh` acts on
#   GH_TOKEN   a token that may open a pull request
#
# Pushes with the credentials of the checkout, which must hold every tag.
# Names a tag it pushes as `tag=v<version>` in GITHUB_OUTPUT, where a runner sets
# one.
#
# Exits 0 when it tags, proposes, or deliberately does neither, and 2 when it
# cannot run.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/../../scripts/report.sh"

if [[ $# -ne 2 ]]; then
  cannot "usage: release.sh <branch> <issue>"
fi

if [[ -z "${GH_REPO:-}" ]]; then
  cannot "GH_REPO is set by the workflow env, and it is empty here."
fi

branch=$1
issue=$2

case "${issue}" in
  '' | 0 | *[!0-9]*)
    cannot "release.sh needs an issue number, and was given '${issue}'."
    ;;
  *) ;;
esac

manifest=Cargo.toml
lock=Cargo.lock
changelog=CHANGELOG.md

author="github-actions[bot]"
email="41898282+github-actions[bot]@users.noreply.github.com"
as_bot=(-c "user.name=${author}" -c "user.email=${email}")

if ! git diff --quiet HEAD --; then
  dirty=$(git --no-pager status --short)
  cannot "release.sh needs a clean tree, because it rewinds to one." "${dirty}"
fi

# The version the `[package]` table names, read from standard input, since only
# that table names purba's own.
read_version() {
  awk '
    /^\[/ { inside = ($0 == "[package]") }
    inside && /^version = "/ { split($0, part, "\""); print part[2]; exit }
  '
}

version=$(read_version <"${manifest}")
if [[ -z "${version}" ]]; then
  cannot "${manifest} names no package version this script can read"
fi

if [[ "${version}" != 0.0.0 ]] &&
  ! git rev-parse --quiet --verify "refs/tags/v${version}" >/dev/null; then
  # The newest commit whose parent named another version, read from the table
  # alone, since a dependency table can name the same version.
  if ! touched=$(git log --format=%H -- "${manifest}"); then
    cannot "the history of ${manifest} could not be read"
  fi
  wrote=""
  for commit in ${touched}; do
    previous=""
    if git cat-file -e "${commit}^:${manifest}" 2>/dev/null; then
      previous=$(git show "${commit}^:${manifest}" | read_version)
    fi
    if [[ "${previous}" != "${version}" ]]; then
      wrote=${commit}
      break
    fi
  done
  echo "tagging v${version} on ${wrote}"
  if ! git "${as_bot[@]}" tag --annotate --message "v${version}" "v${version}" "${wrote}"; then
    cannot "v${version} could not be tagged"
  fi
  if ! git push origin "refs/tags/v${version}"; then
    git tag --delete "v${version}" >/dev/null
    cannot "v${version} could not be pushed"
  fi
  [[ -z "${GITHUB_OUTPUT:-}" ]] || echo "tag=v${version}" >>"${GITHUB_OUTPUT}"
  exit 0
fi

# ⚠️ Never replace this with a force-push. A pull request that broke keeps
# the head that refused it, and the logs hanging off that head.
if ! open=$(gh pr list --repo "${GH_REPO}" --head "${branch}" --state open --json number --jq \
  '.[].number'); then
  cannot "the open pull requests on ${branch} could not be read."
fi
if [[ -n "${open}" ]]; then
  echo "pull request #${open} is already proposing a release on ${branch}, so this run stands down"
  exit 0
fi

if ! next=$(git-cliff --bumped-version); then
  cannot "git-cliff could not compute the next version"
fi
if [[ "${next}" == "v${version}" ]]; then
  echo "nothing since v${version} is released, so there is nothing to propose"
  exit 0
fi
next=${next#v}

echo "proposing v${next}, which follows ${version}"

before=$(git rev-parse HEAD)
restore() {
  git reset --quiet --hard "${before}"
}

sed -i "/^\[package\]$/,/^\[/ s/^version = \"${version}\"$/version = \"${next}\"/" "${manifest}"
sed -i "/^name = \"purba\"$/{n;s/^version = \".*\"$/version = \"${next}\"/}" "${lock}"
written=$(read_version <"${manifest}")
if [[ "${written}" != "${next}" ]] || git diff --quiet -- "${lock}"; then
  restore
  cannot "the version could not be written into ${manifest} and ${lock}"
fi

if ! git-cliff --bump --output "${changelog}"; then
  restore
  cannot "git-cliff could not write ${changelog}"
fi

subject="chore(release): cut v${next} (#${issue})"

# ⚠️ No `-s`. CONTRIBUTING.md: a machine never writes that trailer.
git add -- "${changelog}"
if ! git "${as_bot[@]}" commit --quiet -m "${subject}" -- \
  "${manifest}" "${lock}" "${changelog}"; then
  restore
  cannot "the release could not be committed"
fi

# The branch outlives a closed pull request, so this replaces it. The caller
# keeps the tree it checked out, whether or not the push lands.
pushed=0
git push --force origin "HEAD:refs/heads/${branch}" || pushed=$?
restore
if [[ ${pushed} -ne 0 ]]; then
  cannot "the proposal could not be pushed to ${branch}"
fi

body=$(
  cat <<BODY
A machine wrote this. It proposes a release and certifies nothing.

| | |
|---|---|
| from | \`${version}\` |
| to | \`${next}\` |
| files | \`${manifest}\`, \`${lock}\` and \`${changelog}\` |

⚠️ **\`Origin\` is red on this branch, and that is correct.** A machine never writes a \
\`Signed-off-by:\` trailer, so a person adds it to this commit:

\`\`\`
git fetch origin ${branch}
git switch --detach FETCH_HEAD
mise run sign-off
git push --force-with-lease origin HEAD:${branch}
\`\`\`

**Merging this cuts no tag.** The merge gives the commit a new SHA, so the next run of \
\`Release\` tags \`v${next}\`, whether the schedule or a dispatch starts it.

Opened by \`.github/workflows/release.yml\`, which \
[#${issue}](https://github.com/${GH_REPO}/issues/${issue}) owns.
BODY
)

if ! gh pr create --repo "${GH_REPO}" --base main --head "${branch}" \
  --title "${subject}" \
  --body "${body}"; then
  cannot "${branch} is pushed, and its pull request could not be opened"
fi
