#!/usr/bin/env bash
# Whether the words of the need parse-test-query find the crates this research names.
#
# The matcher is the one scripts/forecast.sh uses at 3093871b: for each word, crates.io's
# list "keyword=<word>&sort=recent-downloads&per_page=20". A crate is found when it is in
# the list of at least one word. The script also prints each named crate's own keywords.
#
#   needs-words.sh            prints a TSV on stdout
set -euo pipefail

words=(parser-combinators grammar combinator)
crates=(nextest-filtering jj-dsl jj-lib cfg-expr cel gherkin unselector kube-core tantivy
  tantivy-query-grammar chumsky winnow nom pest lalrpop peg evalexpr boolean_expression whereexpr)
agent='selection-languages-research'

ask() {
  curl -sS -f -A "${agent}" "$1"
  sleep 1.1
}

found=''
for word in "${words[@]}"; do
  names=$(ask "https://crates.io/api/v1/crates?keyword=${word}&sort=recent-downloads&per_page=20" |
    jq -r '.crates[].name')
  printf 'word\t%s\t%s\n' "${word}" "$(tr '\n' ' ' <<<"${names}")"
  found+=" ${names//$'\n'/ } "
done

printf 'crate\tversion\tfound by the words\tits keywords\n'
for crate in "${crates[@]}"; do
  meta=$(ask "https://crates.io/api/v1/crates/${crate}")
  version=$(jq -r '.crate.max_stable_version // .crate.max_version' <<<"${meta}")
  keywords=$(jq -r '[.crate.keywords[]?] | join(",")' <<<"${meta}")
  if [[ "${found}" == *" ${crate} "* ]]; then hit=yes; else hit=no; fi
  printf '%s\t%s\t%s\t%s\n' "${crate}" "${version}" "${hit}" "${keywords:-none}"
done
