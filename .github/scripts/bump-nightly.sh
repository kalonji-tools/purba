#!/usr/bin/env bash
# Propose a newer nightly for a person to sign.
#
#   the pin:       .config/mise.toml, under `[tools]`
#   the decision:  docs/decisions/the-nightly-is-named-by-a-date.md
#   what you owe:  CONTRIBUTING.md
#
#   bump-nightly.sh <branch> <issue>
#
#   GH_REPO    the repository `gh` acts on
#   GH_TOKEN   a token that may open a pull request
#
# Exits 0 when it proposes one, 0 when it deliberately proposes nothing, 1
# when it refuses what it found, and 2 when it cannot run.
set -euo pipefail

# shellcheck source=.github/scripts/ask-for-sign-off.sh
. "$(dirname "$0")/ask-for-sign-off.sh"

sign_off_start bump-nightly.sh .github/workflows/bump.yml "$@"
stand_down_while_open "a nightly"

# The pin lives in these two, and nothing else here may be committed.
toml=.config/mise.toml
lock=.config/mise.lock

read_pin() {
  sed -n 's/^rust = .*version = "\([^"]*\)".*/\1/p' "${toml}"
}

was=$(read_pin)
if [[ -z "${was}" ]]; then
  cannot "${toml} does not name a rust version this script can read"
fi

if ! mise upgrade --bump rust; then
  cannot "mise could not upgrade the pinned nightly"
fi

# ⚠️ Read the tree, never the exit code.
if git diff --quiet -- "${toml}" "${lock}"; then
  echo "the pinned nightly is the newest mise offers, so there is nothing to propose"
  exit 0
fi

now=$(read_pin)
if [[ -z "${now}" ]]; then
  cannot "${toml} no longer names a rust version this script can read"
fi

if [[ "${now}" = "${was}" ]]; then
  echo "the pin is still ${was}, so there is nothing to propose"
  exit 0
fi

echo "proposing ${now}, which replaces ${was}"

ask_for_sign_off "chore: move the nightly to ${now#nightly-}" "a compiler" \
  "${was}" "${now}" \
  "**If \`Build\` is red, this nightly broke purba.** Leave this pull request open and nothing \
bumps until somebody closes it: the weekly run stands down while it is here, so the compiler that \
broke keeps the head that refused it." \
  "${toml}" "${lock}"
