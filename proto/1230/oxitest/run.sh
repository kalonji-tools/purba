#!/usr/bin/env bash
# The control of #1230: oxitest's -E on each scenario of ../scenarios.sh. It
# copies ast.rs, compile.rs, eval.rs and resource.rs from an oxitest commit,
# unchanged, and builds them with main.rs. This runs oxitest's parser and
# evaluator over the tests of entries.tsv, not the oxitest binary.
# Prototype; this branch is never merged.
#
#   proto/1230/oxitest/run.sh <oxitest checkout> <empty dir> <out dir> [commit]
set -uo pipefail
repo=$1
work=$2
out=$3
rev=${4:-731c2b3f}
here=$(cd "$(dirname "$0")" && pwd)
# shellcheck source=../scenarios.sh
source "$here/../scenarios.sh"
mkdir -p "$work/src/query" "$out"
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
(cd "$work" && cargo build -q --release 2>&1 | grep -E '^error' || true)
bin=$work/target/release/oxq
head_line="oxitest at $(git -C "$repo" rev-parse "$rev")"
for i in "${!KEYS[@]}"; do
  key=${KEYS[$i]}
  expr=${OXITEST[$i]}
  file=$out/$key.txt
  if [[ -z $expr && $key != empty-query ]]; then
    printf '%s\nscenario: %s\n-E: none; oxitest has no form for this row\n' "$head_line" "$key" >"$file"
    continue
  fi
  shown=$expr
  ((${#shown} > 160)) && shown="${expr:0:80} … ${expr: -60} (${#expr} characters)"
  result=$(printf '%s' "$expr" | "$bin" "$here/entries.tsv" 2>&1)
  code=$?
  {
    echo "$head_line"
    echo "scenario: $key"
    echo "-E: $shown"
    echo "--- output"
    printf '%s\n' "$result" | head -20
    echo "--- exit $code"
  } >"$file"
done
echo "control runs in $out"
