#!/usr/bin/env bash
# Builds and runs each probe against the same levels, from `probes/`.
# Usage, from the root of the worktree: research/configuration-crates/probes/run.sh > research/configuration-crates/probes/run.out
set -euo pipefail
cd "$(dirname "$0")"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$(mktemp -d)}
echo "# head $(git rev-parse HEAD); $(cargo --version)"
for package in config figment tier; do
  cargo build -q --locked --manifest-path "${package}/Cargo.toml"
done
for bin in probe-config figment_rs figment2_rs probe-tier; do
  echo "\$ env -i PATH=\"\$PATH\" PURBA_TIMEOUT=2 PURBA_WORKERS=2 ${bin}"
  env -i PATH="${PATH}" PURBA_TIMEOUT=2 PURBA_WORKERS=2 "${CARGO_TARGET_DIR}/debug/${bin}"
done
