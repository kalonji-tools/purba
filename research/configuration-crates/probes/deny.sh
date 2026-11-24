#!/usr/bin/env bash
# Runs purba's advisory, licence and source checks on the lock of each probe package.
# Usage, from the root of the worktree: mise exec -- research/configuration-crates/probes/deny.sh
set -uo pipefail
root=$(git rev-parse --show-toplevel)
here=$(dirname "$0")
for package in config figment tier star-toml; do
  for which in advisories licenses sources; do
    echo "\$ cargo-deny --config .cargo/deny.toml --manifest-path probes/${package}/Cargo.toml check ${which}"
    cargo-deny --config "${root}/.cargo/deny.toml" --manifest-path "${here}/${package}/Cargo.toml" check "${which}" 2>&1 |
      grep -E '^(error|warning)\[|^ +[├└]|^ *= (ID|license|[A-Z][a-z]+:)|(advisories|licenses|sources) (ok|FAILED)' |
      sed "s#${root}/##g"
  done
done
