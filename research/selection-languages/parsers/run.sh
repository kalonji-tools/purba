#!/usr/bin/env bash
# Probe five parser crates with one grammar: chumsky, winnow, nom, pest and lalrpop. The
# grammar is in src/main.rs, src/grammar/query.pest and src/grammar/query.lalrpop. Each case
# prints the tree or the crate's error. A nesting case prints how far each stage got. The
# probe forgets the tree after a parse, so its recursive drop does not count.
# Usage: bash run.sh <empty dir>
set -uo pipefail
work=$1
here=$(cd "$(dirname "$0")" && pwd)
cp -r "$here/Cargo.toml" "$here/Cargo.lock" "$here/build.rs" "$here/src" "$work/"
cd "$work" || exit 1
cargo --version
cargo build --release --locked -q 2>&1
bin=$work/target/release/p-parsers

ask() {
  printf -- '--- %s: %s\n' "$1" "$2"
  "$bin" "$1" "$2" 2>/dev/null
  printf 'exit %s\n' "$?"
}

nest() {
  awk -v n="$1" -v pre="$2" -v mid="$3" -v post="$4" \
    'BEGIN{s="";for(i=0;i<n;i++)s=s pre;s=s mid;for(i=0;i<n;i++)s=s post;printf "%s",s}'
}

deep() {
  local crate=$1 pre=$2 mid=$3 post=$4 n out code
  for n in 100 1000 3000 10000 30000 100000; do
    out=$(nest "$n" "$pre" "$mid" "$post" | "$bin" "$crate" - 2>&1)
    code=$?
    printf '%s, [%s] x %s: exit %s; %s\n' "$crate" "$pre" "$n" "$code" \
      "$(printf '%s\n' "$out" | grep -o 'stage: [a-z ]*\|overflowed its stack\|stack limit reached' | sort -u | tr '\n' ' ')"
  done
}

for crate in chumsky winnow nom pest lalrpop; do
  echo "== ${crate}"
  ask "$crate" 'store:: & tag(slow) & !tag(flaky)'
  ask "$crate" 'tag(db) | tag(slow) & !tag(db)'
  ask "$crate" 'store::migrate'
  ask "$crate" 'tests/api'
  ask "$crate" 'tag(slow) &'
  ask "$crate" 'tag(slow) & (tag(db)'
  ask "$crate" 'tag(slow) && tag(db)'
done

echo '== nesting'
for crate in chumsky winnow nom pest lalrpop; do
  deep "$crate" '(' 'tag(slow)' ')'
  deep "$crate" '!' 'tag(slow)' ''
done
