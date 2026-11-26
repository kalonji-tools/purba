#!/usr/bin/env bash
# Run the bats tests on the wheel of each mutant, one at a time: apply its diff,
# run `mise run test:wheel`, print each bats test that failed, and revert.
# usage: bats.sh <log dir> <diff>...   (run from the root of a purba checkout)
# .cargo/config.toml denies warnings. CARGO_BUILD_WARNINGS=allow lets a mutant that
# leaves a variable unused build, as `--cap-lints=true` did under cargo-mutants.
set -u
logs=$1
shift
mkdir -p "$logs"
export CARGO_BUILD_WARNINGS=allow
for diff in "$@"; do
  name=$(basename "$diff" .diff)
  # A diff of cargo-mutants names the file on its first line and the mutant on
  # its second, so `git apply` refuses it; `patch` takes the file by name.
  file=$(sed -n '1s/^--- //p' "$diff")
  patch --quiet "$file" "$diff" || {
    echo "$name	apply-failed"
    continue
  }
  # A mutant can loop forever; 900 s is over seven times the 2 min of a clean run.
  timeout 900 mise run test:wheel >"$logs/$name.log" 2>&1
  status=$?
  patch --quiet -R "$file" "$diff" || exit 99
  git diff --quiet -- "$file" || exit 98
  failed=$(grep -c '^not ok' "$logs/$name.log")
  first=$(grep '^not ok' "$logs/$name.log" | head -3 | tr '\n' '|')
  # `-` for none: the hook that trims trailing whitespace would drop a trailing tab.
  echo "$name	exit=$status	not-ok=$failed	${first:--}"
done
