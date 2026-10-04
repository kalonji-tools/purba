#!/usr/bin/env bash
# Say which step of a release `main` owes. It gates nothing.
#
#   the caller:    .github/workflows/release-owed.yml
#   the steps:     .github/scripts/release.sh
#   the register:  cliff.toml
#
#   release-owed.sh
#
# Prints the report as Markdown. Exits 0 on each answer, and 2 when it cannot
# compute one, which is a different thing from a release being owed.
set -euo pipefail

if [[ $# -ne 0 ]]; then
  echo "usage: release-owed.sh" >&2
  exit 2
fi

manifest=Cargo.toml

# Only the `[package]` table names purba's own version.
version=$(awk '
  /^\[/ { inside = ($0 == "[package]") }
  inside && /^version = "/ { split($0, part, "\""); print part[2]; exit }
' "${manifest}")
if [[ -z "${version}" ]]; then
  echo "::error::${manifest} names no package version this script can read" >&2
  exit 2
fi

if [[ "${version}" != 0.0.0 ]] &&
  ! git rev-parse --quiet --verify "refs/tags/v${version}" >/dev/null; then
  echo "**A tag is owed.** \`${manifest}\` names \`${version}\`, and no tag carries it."
  echo "Dispatch \`Release\`, and it tags \`v${version}\`."
  exit 0
fi

# With nothing to bump, git-cliff prints the current tag and exits 0.
next=$(git-cliff --bumped-version) || next=""
if [[ -z "${next}" ]]; then
  echo "::error::git-cliff could not compute the next version" >&2
  exit 2
fi
if [[ "${next}" == "v${version}" ]]; then
  echo "No release is owed. Nothing since \`v${version}\` reaches the changelog."
  exit 0
fi

if ! held=$(git-cliff --unreleased --bump --strip all); then
  echo "::error::git-cliff could not render what the release holds" >&2
  exit 2
fi

echo "**A release is owed**, from \`${version}\` to \`${next#v}\`."
echo "Dispatch \`Release\`, and it proposes the release."
echo
echo "${held}"
