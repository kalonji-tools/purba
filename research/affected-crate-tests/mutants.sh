#!/usr/bin/env bash
# Run cargo-mutants on one crate, test each mutant against every Rust test of
# the workspace, and go on past the first target that fails, so the log of each
# mutant names every target that kills it.
# usage: mutants.sh <cargo-mutants> <crate> <out dir> <tmp dir> [more cargo-mutants args...]
#   (run from the root of a purba checkout, under `mise exec --`)
# `--cap-lints=true`: .cargo/config.toml denies warnings, so a mutant that leaves a
# variable unused would not build and would count as unviable.
set -u
mutants=$1 crate=$2 out=$3 tmp=$4
shift 4
LD_LIBRARY_PATH=$(python -c 'import sysconfig; print(sysconfig.get_config_var("LIBDIR"))')
export LD_LIBRARY_PATH
mkdir -p "$out" "$tmp"
TMPDIR=$tmp "$mutants" mutants -p "$crate" --test-workspace=true --cap-lints=true \
  --cargo-test-arg=--no-fail-fast --jobs 4 --minimum-test-timeout 120 \
  --output "$out" "$@"
