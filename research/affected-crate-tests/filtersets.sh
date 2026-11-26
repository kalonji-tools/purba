#!/usr/bin/env bash
# List, without running them, the tests that each cargo-nextest filterset selects
# for an edit to one crate. Prints, per expression, the count of tests and the
# count of each (crate, kind) of binary.
# usage: filtersets.sh <crate>   (cargo-nextest on PATH; run from the root of a
#   purba checkout, under `mise exec --`)
set -u
crate=$1
LD_LIBRARY_PATH=$(python -c 'import sysconfig; print(sysconfig.get_config_var("LIBDIR"))')
export LD_LIBRARY_PATH
for expr in \
  'all()' \
  "package($crate)" \
  "rdeps($crate)" \
  "rdeps($crate) - kind(lib)" \
  "package($crate) | (rdeps($crate) - kind(lib))"; do
  echo "---- -E '$expr'"
  cargo nextest list --workspace --message-format oneline -E "$expr" 2>/dev/null >"${TMPDIR:-.}/list.txt"
  echo "tests: $(wc -l <"${TMPDIR:-.}/list.txt")"
  # A oneline row is `<package>[::<binary>] <test>`; `::` marks a binary that is not the lib.
  awk '{split($1, a, "::"); kind = (a[2] == "") ? "lib" : "test " a[2]; print a[1] "\t" kind}' \
    "${TMPDIR:-.}/list.txt" | sort | uniq -c | awk '{printf "  %s", $0 "\n"}' | sort -k2
done
