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
# Exits 0 when it tags, proposes, or deliberately does neither, 1 when it
# refuses what it found, and 2 when it cannot run.
set -euo pipefail

# shellcheck source=.github/scripts/ask-for-sign-off.sh
. "$(dirname "$0")/ask-for-sign-off.sh"

sign_off_start release.sh .github/workflows/release.yml "$@"

manifest=Cargo.toml
lock=Cargo.lock
changelog=CHANGELOG.md

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

stand_down_while_open "a release"

if ! next=$(git-cliff --bumped-version); then
  cannot "git-cliff could not compute the next version"
fi
if [[ "${next}" == "v${version}" ]]; then
  echo "nothing since v${version} is released, so there is nothing to propose"
  exit 0
fi
next=${next#v}

echo "proposing v${next}, which follows ${version}"

sed -i "/^\[package\]$/,/^\[/ s/^version = \"${version}\"$/version = \"${next}\"/" "${manifest}"
sed -i "/^name = \"purba\"$/{n;s/^version = \".*\"$/version = \"${next}\"/}" "${lock}"
written=$(read_version <"${manifest}")
if [[ "${written}" != "${next}" ]] || git diff --quiet -- "${lock}"; then
  cannot "the version could not be written into ${manifest} and ${lock}"
fi

if ! git-cliff --bump --output "${changelog}"; then
  cannot "git-cliff could not write ${changelog}"
fi

ask_for_sign_off "chore(release): cut v${next} (#${issue})" "a release" "${version}" "${next}" \
  "**Merging this cuts no tag.** The merge gives the commit a new SHA, so the next run of \
\`Release\` tags \`v${next}\`, whether the schedule or a dispatch starts it." \
  "${manifest}" "${lock}" "${changelog}"
