#!/usr/bin/env bash
# The lines that the probe grammar took in each parser crate. A row counts every line of the
# crate's module in parsers/src/main.rs, from "mod with_<crate> {" to its closing "}", plus
# every line of the crate's grammar file. The shared parts (the tree, main) are not counted.
#
#   sizes.sh            prints a TSV on stdout
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
main=$here/parsers/src/main.rs

printf 'crate\tmodule lines\tgrammar file\tgrammar lines\ttotal\n'
for crate in chumsky winnow nom pest lalrpop; do
  module=$(awk -v m="mod with_${crate} {" '$0==m{n=0;on=1} on{n++} on&&/^}/{print n;on=0}' "$main")
  case $crate in
    pest) file=parsers/src/grammar/query.pest ;;
    lalrpop) file=parsers/src/grammar/query.lalrpop ;;
    *) file='' ;;
  esac
  grammar=0
  [[ -n "$file" ]] && grammar=$(wc -l <"$here/$file")
  printf '%s\t%s\t%s\t%s\t%s\n' "$crate" "$module" "${file:-none}" "$grammar" "$((module + grammar))"
done
