#!/usr/bin/env bash
# Count the dials in dials/*.tsv, and print the tables of the findings file.
# usage: count.sh [tables|appendix]
set -eu
here=$(cd "$(dirname "$0")" && pwd)
runners=(pytest unittest nextest go vitest jest)
mode=${1:-tables}

awk_common='BEGIN { FS = "\t" } FNR == 1 { next }'

if [ "$mode" = tables ]; then
  echo "### Dials per runner"
  echo
  printf '| |'
  for f in "${runners[@]}"; do printf ' %s |' "$f"; done
  echo
  printf '|---|'
  for _ in "${runners[@]}"; do printf -- '---|'; done
  echo
  printf '| dials |'
  for f in "${runners[@]}"; do printf ' %s |' "$(awk "$awk_common"' { n++ } END { print n + 0 }' "$here/dials/$f.tsv")"; done
  echo
  echo
  echo "### Dials per group"
  echo
  printf '| group |'
  for f in "${runners[@]}"; do printf ' %s |' "$f"; done
  echo
  printf '|---|'
  for _ in "${runners[@]}"; do printf -- '---|'; done
  echo
  for g in sel ord par tmo rep stp chk fix imp ast out cap cch plg dbg cov bld cfg; do
    printf '| %s |' "$g"
    for f in "${runners[@]}"; do
      printf ' %s |' "$(awk "$awk_common"' $2 == g { n++ } END { print n + 0 }' g="$g" "$here/dials/$f.tsv")"
    done
    echo
  done
  echo
  echo "### Dials per class"
  echo
  printf '| class |'
  for f in "${runners[@]}"; do printf ' %s |' "$f"; done
  echo
  printf '|---|'
  for _ in "${runners[@]}"; do printf -- '---|'; done
  echo
  for c in F T B L; do
    printf '| %s |' "$c"
    for f in "${runners[@]}"; do
      printf ' %s |' "$(awk "$awk_common"' $4 == c { n++ } END { print n + 0 }' c="$c" "$here/dials/$f.tsv")"
    done
    echo
  done
  for row in "↕ (both ways)#\$5 == \"↕\"" "changes a test's outcome#\$6 == \"test\"" "changes only the run#\$6 == \"run\"" "L, or ↕, that changes a test's outcome#(\$4 == \"L\" || \$5 == \"↕\") && \$6 == \"test\""; do
    label=${row%%#*}
    cond=${row#*#}
    printf '| %s |' "$label"
    for f in "${runners[@]}"; do
      printf ' %s |' "$(awk "$awk_common $cond"' { n++ } END { print n + 0 }' "$here/dials/$f.tsv")"
    done
    echo
  done
  echo
  echo "### Dials per level"
  echo
  printf '| level |'
  for f in "${runners[@]}"; do printf ' %s |' "$f"; done
  echo
  printf '|---|'
  for _ in "${runners[@]}"; do printf -- '---|'; done
  echo
  for l in C E P K U; do
    printf '| %s |' "$l"
    for f in "${runners[@]}"; do
      printf ' %s |' "$(awk "$awk_common"' { if (index(" " $3 " ", " " l " ")) n++ } END { print n + 0 }' l="$l" "$here/dials/$f.tsv")"
    done
    echo
  done
  for row in "C only" "two levels or more"; do
    printf '| %s |' "$row"
    for f in "${runners[@]}"; do
      if [ "$row" = "C only" ]; then
        printf ' %s |' "$(awk "$awk_common"' $3 == "C" { n++ } END { print n + 0 }' "$here/dials/$f.tsv")"
      else
        printf ' %s |' "$(awk "$awk_common"' split($3, a, " ") >= 2 { n++ } END { print n + 0 }' "$here/dials/$f.tsv")"
      fi
    done
    echo
  done
else
  for f in "${runners[@]}"; do
    echo "### $f"
    echo
    echo "| dial | group | levels | class | outcome | note |"
    echo "|---|---|---|---|---|---|"
    awk "$awk_common"' {
      d = $1; gsub(/\|/, "\\|", d); n = $7; gsub(/\|/, "\\|", n)
      printf "| `%s` | %s | %s | %s%s | %s | %s |\n", d, $2, $3, $4, ($5 == "" ? "" : " " $5), $6, n
    }' "$here/dials/$f.tsv"
    echo
  done
fi
