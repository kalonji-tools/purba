#!/usr/bin/env bash
# Probe five language crates on one suite: cfg-expr (Cargo's cfg()), cel (CEL), gherkin
# (Cucumber's tag expressions), unselector with kube-core (Kubernetes' label selectors), and
# evalexpr (a boolean evaluator). Each case prints the crate's tree or error, the tests it
# selects, and the exit code. A nesting case prints how far each stage got.
# Usage: bash run.sh <empty dir>
set -uo pipefail
work=$1
here=$(cd "$(dirname "$0")" && pwd)
cp -r "$here/Cargo.toml" "$here/Cargo.lock" "$here/src" "$work/"
cd "$work" || exit 1
cargo --version
cargo build --release --locked -q 2>&1
bin=$work/target/release/p-small

ask() {
  printf -- '--- %s: %s\n' "$1" "$2"
  "$bin" "$1" "$2" 2>/dev/null
  printf 'exit %s\n' "$?"
}

# nest <n> <before> <middle> <after>: <before> n times, <middle>, then <after> n times.
nest() {
  awk -v n="$1" -v pre="$2" -v mid="$3" -v post="$4" \
    'BEGIN{s="";for(i=0;i<n;i++)s=s pre;s=s mid;for(i=0;i<n;i++)s=s post;printf "%s",s}'
}

deep() {
  local lang=$1 pre=$2 mid=$3 post=$4 n out code
  for n in 100 1000 10000 30000 100000; do
    out=$(nest "$n" "$pre" "$mid" "$post" | "$bin" "$lang" - 2>&1)
    code=$?
    printf '%s, [%s] x %s: exit %s; %s\n' "$lang" "$pre" "$n" "$code" \
      "$(printf '%s\n' "$out" | grep -o 'stage: [a-z]*\|overflowed its stack\|Recursion limit of [0-9]* exceeded' | sort -u | tr '\n' ' ')"
  done
}

echo '== the example: the slow tests of store, but not the flaky ones'
ask cfg-expr 'all(module = "store", slow, not(flaky))'
ask cel 'module == "store" && "slow" in tags && !("flaky" in tags)'
ask gherkin '@store and @slow and not @flaky'
ask k8s 'module=store,slow,!flaky'
ask evalexpr 'module == "store" && slow && !flaky'

echo '== precedence: db, or slow and not db. AND before OR selects 5; left to right selects 3'
ask cfg-expr 'any(db, all(slow, not(db)))'
ask cel '"db" in tags || "slow" in tags && !("db" in tags)'
ask gherkin '@db or @slow and not @db'
ask evalexpr 'db || slow && !db'
echo '-- a label selector has no OR: the comma is AND, and a set is one key'
ask k8s 'module in (store,math),slow'
ask k8s 'db|slow'

echo '== an id of purba, and a path, as the whole query'
for lang in cfg-expr cel gherkin k8s evalexpr; do
  ask "$lang" 'store::migrate'
  ask "$lang" 'tests/api'
done

echo '== an error at a position: a missing operand, and an unclosed parenthesis'
ask cfg-expr 'all(slow,'
ask cfg-expr 'all(slow, not(db)'
ask cel '"slow" in tags &&'
ask cel '"slow" in tags && ("db" in tags'
ask gherkin '@slow and'
ask gherkin '@slow and (@db'
ask k8s 'slow,!'
ask k8s 'module in (store'
ask evalexpr 'slow &&'
ask evalexpr 'slow && (db'

echo '== nesting'
deep cfg-expr 'all(' 'slow' ')'
deep cfg-expr 'not(' 'slow' ')'
deep cel '(' '"slow" in tags' ')'
deep cel '("slow" in tags && ' '"slow" in tags' ')'
deep cel '!' '"slow" in tags' ''
deep gherkin '(' '@slow' ')'
deep gherkin '(@slow and ' '@slow' ')'
deep gherkin 'not ' '@slow' ''
deep evalexpr '(' 'slow' ')'
deep evalexpr '(slow && ' 'slow' ')'
deep evalexpr '!' 'slow' ''

echo '== width: n function predicates side by side in one all() or any(). cfg-expr counts them in a u8.'
echo '-- right answers: any(...) selects 6; all(...) selects 3'
for n in 255 256 257; do
  for f in any all; do
    q="$f($(nest "$n" 'not(flaky), ' '' '')slow)"
    printf '%s, %s x not(flaky), slow: %s\n' "$f" "$n" "$(printf '%s' "$q" | "$bin" cfg-expr - 2>/dev/null | tr '\n' ' ')"
  done
done

echo '== cel: the error of a deep query, printed. Its caret is padded to the error column.'
for n in 60000 70000; do
  out=$(nest "$n" '(' '"slow" in tags' ')' | "$bin" cel - 2>&1)
  code=$?
  printf 'cel, [(] x %s: exit %s; %s\n' "$n" "$code" \
    "$(printf '%s\n' "$out" | grep -o 'Recursion limit of [0-9]* exceeded\|panicked at [^ ]*\|Formatting argument out of range' | sed 's#.*/cel-#cel-#' | sort -u | tr '\n' ' ')"
done
