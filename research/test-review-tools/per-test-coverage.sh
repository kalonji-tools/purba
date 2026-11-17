#!/usr/bin/env bash
# Coverage of each test alone, for one crate, and the tests whose covered lines
# another test's covered lines contain. Research for #1058; not wired to any task.
#
#   nix-shell -p cargo-llvm-cov --run 'mise exec -- bash per-test-coverage.sh <package> <out-dir>'
#
# Needs: cargo-llvm-cov, llvm-cov and llvm-profdata of the toolchain's LLVM (in
# LLVM_COV and LLVM_PROFDATA), jq. LD_LIBRARY_PATH must hold libpython's directory,
# as test:rust sets it.
set -o errexit -o nounset -o pipefail
pkg=$1 out=$2
mkdir -p "${out}"
cargo llvm-cov clean --workspace
# One instrumented build; `--no-report` keeps the binaries for the runs below.
cargo llvm-cov --no-report --package "${pkg}" --all-targets >"${out}/build.log" 2>&1
# `--list` names each test as `<name>: test`, one binary at a time.
cargo llvm-cov --no-report --package "${pkg}" --all-targets -- --list --format terse 2>/dev/null |
  sed -n 's/: test$//p' | sort -u >"${out}/tests.txt"
start=$(date +%s%N)
while read -r t; do
  cargo llvm-cov clean --profraw-only >/dev/null 2>&1
  # A test name can exist in one binary only; the other binaries run no test.
  cargo llvm-cov --no-report --package "${pkg}" --all-targets -- --exact "${t}" >/dev/null 2>&1
  # Covered lines of the crate's own src/, one "file:line" for each: the line
  # where a region with a non-zero count starts. A segment is
  # [line, column, count, has-count, is-region-entry, is-gap].
  cargo llvm-cov report --json --package "${pkg}" 2>/dev/null |
    jq -r --arg src "/${pkg#purba-}/src/" '
      .data[0].files[] | select(.filename | contains($src)) | .filename as $f
      | .segments[] | select(.[2] > 0 and .[3] and .[4] and (.[5] | not))
      | "\($f):\(.[0])"' |
    sort -u >"${out}/lines.$(printf '%s' "${t}" | tr -c 'A-Za-z0-9_' '_')"
done <"${out}/tests.txt"
ms=$((($(date +%s%N) - start) / 1000000))
tests=$(wc -l <"${out}/tests.txt")
echo "per-test runs: ${ms} ms for ${tests} tests"
# A test is covered-subsumed when another test covers every line it covers.
cd "${out}"
for a in lines.*; do
  [[ -s "${a}" ]] || continue
  for b in lines.*; do
    [[ "${a}" == "${b}" ]] && continue
    # The lines of `a` that `b` does not cover; none means `b` covers them all.
    only_a=$(comm -23 "${a}" "${b}")
    [[ -z "${only_a}" ]] || continue
    kind=inside
    if cmp -s "${a}" "${b}"; then kind=same; fi
    echo "${kind} ${a#lines.} ${b#lines.}"
  done
done >subsumed.txt
empty=$(find . -name 'lines.*' -empty -printf x)
inside=$(awk '{print $2}' subsumed.txt | sort -u)
same=$(awk '$1 == "same"' subsumed.txt)
echo "tests with no covered line: ${#empty}"
echo "tests whose lines another test covers: $(grep -c . <<<"${inside}" || true)"
echo "pairs with the same lines, each pair twice: $(grep -c . <<<"${same}" || true)"
