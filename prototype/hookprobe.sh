#!/usr/bin/env bash
# Re-runs the git-hook measurement in prototype/README.md §3.
#
# It gives both arms one environment: git, mise and just on PATH, and NO cargo.
# That is what a commit launched from an editor gets, and it is the only
# surface on which the two runners behave differently by construction.
#
# Usage: prototype/hookprobe.sh /path/to/mise
set -euo pipefail

MISE="${1:?pass the path to a mise >= 2026.9.7}"
cd "$(git rev-parse --show-toplevel)"

PREK="$("$MISE" which prek)"
HOOKPATH="$(dirname "$MISE"):$(dirname "$("$MISE" which just)"):$(dirname "$(command -v git)"):/usr/bin:/bin"

printf 'cargo on the hook PATH: %s\n\n' \
  "$(PATH="$HOOKPATH" command -v cargo || echo 'absent — as intended')"

for arm in a b; do
  rm -f tasks.toml justfile
  case "$arm" in
    a) name='ARM A: mise run fmt:check'; entry='mise run fmt:check'
       cp prototype/arm-a/tasks.toml tasks.toml ;;
    b) name='ARM B: just fmt-check';     entry='just fmt-check'
       cp prototype/arm-b/justfile justfile ;;
  esac

  cat > prek.toml <<EOF
[[repos]]
repo = "local"
[[repos.hooks]]
id = "gate"
name = "$name"
entry = "$entry"
language = "system"
pass_filenames = false
files = '\\.rs\$'
EOF

  printf -- '--- %s\n' "$name"
  # env -i so nothing of the caller's shell leaks in and flatters an arm.
  env -i HOME="$HOME" PATH="$HOOKPATH" "$PREK" run --all-files 2>&1 | tail -4
  printf '\n'
done

rm -f tasks.toml justfile prek.toml
