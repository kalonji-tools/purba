#!/usr/bin/env bash
# compare.sh LEVELS_TSV
# Compares the levels that levels.sh printed with baseline.tsv (research/future-crates.md, B7).
# Prints, for each input, how many of the crates agree, differ, or get no level from the script,
# then each difference as "crate input file-level script-level".
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
awk -F'\t' '
  BEGIN { split("blast upkeep maintainers review tests unsafe churn owner", name, " ") }
  FNR == NR { if ($1 !~ /^#/) for (i = 3; i <= 10; i++) base[$1, i] = $i; next }
  {
    for (i = 3; i <= 10; i++) {
      row = sprintf("%s\t%s\t%s\t%s\n", $1, name[i - 2], base[$1, i], $i)
      if ($i == "-") { none[i]++; diffs = diffs row }
      else if ($i == base[$1, i]) agree[i]++
      else { differ[i]++; diffs = diffs row }
    }
    n++
  }
  END {
    printf "input\tagree\tdiffer\tno level\n"
    for (i = 3; i <= 10; i++) {
      printf "%s\t%d\t%d\t%d\n", name[i - 2], agree[i], differ[i], none[i]
      ta += agree[i]; td += differ[i]; tn += none[i]
    }
    printf "all\t%d\t%d\t%d\n", ta, td, tn
    printf "crates\t%d\n\ncrate\tinput\tfile\tscript\n%s", n, diffs
  }
' "${here}/baseline.tsv" "$1"
