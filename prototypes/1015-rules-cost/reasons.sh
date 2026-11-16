#!/usr/bin/env bash
# PROTOTYPE for #1015, never merge. Count the tests that each reason of main's
# selection refuses, from a tally; a test counts once for each reason.
set -euo pipefail
awk -F'\t' '$4 == "refused" {
  n = split($5, r, "; ")
  delete seen
  for (i = 1; i <= n; i++) {
    k = r[i]
    if (k ~ /decorator/) k = "foreign-decorator"
    else if (k ~ /no default/) k = "param-no-default"
    else if (k ~ /async/) k = "async"
    else if (k ~ /generator/) k = "generator"
    else if (k ~ /returns a value/) k = "returned-value"
    else if (k ~ /again/) k = "name-bound-again"
    else if (k ~ /with no def/) k = "name-without-def"
    else if (k ~ /sits under/) k = "def-under-block"
    else if (k ~ /not one comparison/) k = "not-one-comparison"
    else if (k ~ /computes/) k = "computed-expected"
    else if (k ~ /private name/) k = "private-name"
    else k = "other"
    if (!(k in seen)) { seen[k] = 1; c[k]++ }
  }
}
END { for (k in c) printf "%5d  %s\n", c[k], k }' "$1" | sort -rn
lines="$(wc -l <"$1")"
printf 'tests read: %s\n' "${lines}"
