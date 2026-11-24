#!/usr/bin/env bash
# Measure the size of each runner's selection language, from source pinned at a
# tag. It prints one row per file: total lines, production lines, test lines.
# Matcher: every line counts, blank lines and comments too. In a Rust file, a
# production line is a line before the first line that is exactly
# `#[cfg(test)]`. A file under a tests/ directory, or named *_test.go,
# test_*.py or *.test.ts, is all test lines.
# Usage: bash sizes.sh <pytest clone at 9.2.0> <nextest clone at cargo-nextest-0.9.149>
#        <vitest clone at v5.1.1> <dir with go1.26.10 files> <oxitest clone>
set -euo pipefail
pytest=$1
nextest=$2
vitest=$3
gosrc=$4
oxitest=$5
row() { printf '%s\t%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" "$5"; }
rust_row() {
  local runner=$1 label=$2 file=$3
  local total cut prod
  total=$(wc -l <"$file")
  cut=$(grep -n -x '#\[cfg(test)\]' "$file" | head -1 | cut -d: -f1 || true)
  if [ -z "$cut" ]; then prod=$total; else prod=$((cut - 1)); fi
  row "$runner" "$label" "$total" "$prod" "$((total - prod))"
}
printf 'runner\tfile\ttotal\tproduction\ttest\n'
for f in src/_pytest/mark/expression.py; do
  n=$(wc -l <"$pytest/$f")
  row pytest "$f" "$n" "$n" 0
done
n=$(wc -l <"$pytest/testing/test_mark_expression.py")
row pytest testing/test_mark_expression.py "$n" 0 "$n"
# proptest_helpers.rs and src/bin/ build only under the feature
# `internal-testing` (nextest-filtering/src/lib.rs L10, Cargo.toml L15-17), so
# they count as test lines.
for f in $(cd "$nextest" && find nextest-filtering/src -name '*.rs' | sort); do
  case $f in
    */proptest_helpers.rs | */bin/*)
      n=$(wc -l <"$nextest/$f")
      row nextest "$f" "$n" 0 "$n"
      ;;
    *) rust_row nextest "$f" "$nextest/$f" ;;
  esac
done
for f in $(cd "$nextest" && find nextest-filtering/tests -name '*.rs' | sort); do
  n=$(wc -l <"$nextest/$f")
  row nextest "$f" "$n" 0 "$n"
done
for f in packages/vitest/src/runtime/runner/utils/tags.ts packages/vitest/src/node/tags.ts; do
  n=$(wc -l <"$vitest/$f")
  row vitest "$f" "$n" "$n" 0
done
n=$(wc -l <"$vitest/test/e2e/test/test-tags.test.ts")
row vitest test/e2e/test/test-tags.test.ts "$n" 0 "$n"
for f in match.go expr.go; do
  n=$(wc -l <"$gosrc/$f")
  row go "$f" "$n" "$n" 0
done
for f in match_test.go expr_test.go; do
  n=$(wc -l <"$gosrc/$f")
  row go "$f" "$n" 0 "$n"
done
tmp=$(mktemp)
for f in ast compile eval resource; do
  git -C "$oxitest" show "731c2b3f:src/query/${f}.rs" >"$tmp"
  rust_row oxitest "src/query/${f}.rs" "$tmp"
done
rm -f "$tmp"
