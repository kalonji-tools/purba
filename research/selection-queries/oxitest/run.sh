#!/usr/bin/env bash
# Probe oxitest's query language. It copies ast.rs, compile.rs, eval.rs and
# resource.rs from an oxitest commit, unchanged, and builds them with main.rs.
# This runs oxitest's parser and evaluator, not the oxitest binary.
# Usage: bash run.sh <oxitest checkout> <empty dir> [commit]
set -uo pipefail
repo=$1
work=$2
rev=${3:-731c2b3f}
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$work/src/query"
for f in ast compile eval resource; do
  git -C "$repo" show "${rev}:src/query/${f}.rs" >"$work/src/query/${f}.rs"
done
cp "$here/main.rs" "$work/src/main.rs"
cat >"$work/Cargo.toml" <<'TOML'
[package]
name = "oxq"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "=4.6.6", features = ["derive"] }
miette = "=7.6.0"
regex = "=1.13.1"
thiserror = "=2.0.20"

[workspace]
TOML
cd "$work" || exit 1
printf 'oxitest at %s\n' "$(git -C "$repo" rev-parse "$rev")"
cargo build -q --release 2>&1 | grep -v '^warning' | grep -E 'error' || true
bin=$work/target/release/oxq
probe() {
  local name=$1
  local expr=$2
  local out
  out=$("$bin" "$expr" 2>&1)
  local code=$?
  printf '### %s: -E %q\nexit %s\n%s\n\n' "$name" "${expr:0:60}" "$code" "$(printf '%s\n' "$out" | head -8)"
}
probe union 'mark(slow) | mark(db)'
probe intersection 'mark(slow) & mark(db)'
probe exclusion 'mark(slow) & !mark(db)'
probe nesting '(mark(slow) | mark(db)) & !(mark(slow) & mark(db))'
probe precedence 'mark(db) | mark(slow) & !mark(db)'
probe precedence-grouped 'mark(db) | (mark(slow) & !mark(db))'
probe name-add 'name(add)'
probe name-file 'name(math)'
probe mark-substring 'mark(slo)'
probe mark-exact 'mark(=slo)'
probe mark-unknown 'mark(nosuchmark)'
probe predicate-unknown 'tag(slow)'
probe syntax-error 'mark(slow) &'
for n in 1000 10000 50000; do
  probe "parens-$n" "$(printf '%*s' "$n" '' | tr ' ' '(')mark(slow)$(printf '%*s' "$n" '' | tr ' ' ')')"
done
for n in 1000 10000 50000; do
  probe "bangs-$n" "$(printf '%*s' "$n" '' | tr ' ' '!')mark(slow)"
done
