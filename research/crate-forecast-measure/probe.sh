#!/usr/bin/env bash
# probe.sh OUT CRATE VERSION
# Run it from a purba checkout under `mise exec --`, so that cargo, clippy-driver and
# PYO3_PYTHON are purba's pinned ones.
#
# Makes one probe crate that depends on CRATE = "=VERSION" with default features, then:
#   1. blast: `cargo tree -e normal,build --target x86_64-unknown-linux-gnu`, distinct
#      "name version", without the probe and without the crate itself (the reading of
#      research/future-crates.md).
#   2. unsafe: `cargo check` with clippy-driver as the wrapper and two forced lints,
#      `unsafe_code` and `clippy::undocumented_unsafe_blocks` (research/crate-gate-tools.md
#      § 3.6). A forced lint is not capped on a dependency, so the warnings cover what the
#      probe compiles.
#   3. src: the directory of the crate's published source, for the `#[test]` count of tests.
# Writes OUT/probe/CRATE/{tree.txt,set.txt,check.json,srcdir.txt}. The target dir is shared.
#
# This builds the crate, so it runs the build scripts and proc macros of the crate and of what
# it brings. Run it only in a scratch directory.
set -euo pipefail
out=$1
c=$2
v=$3
P=${out}/probe/${c}
T=${out}/target
TRIPLE=x86_64-unknown-linux-gnu
rm -rf "${P}"
mkdir -p "${P}/src" # cargo makes ${T} itself, so that `cargo clean` accepts it
: >"${P}/src/lib.rs"
cat >"${P}/Cargo.toml" <<EOF
[package]
name = "probe_${c//-/_}"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
${c} = "=${v}"

[workspace]
EOF
M=${P}/Cargo.toml
cargo generate-lockfile -q --manifest-path "${M}"
cargo tree -q --manifest-path "${M}" -e normal,build --prefix none --target "${TRIPLE}" \
  >"${P}/tree.txt"
awk '{print $1" "$2}' "${P}/tree.txt" | sort -u | grep -v '^probe_' |
  grep -vxF "${c} v${v}" >"${P}/set.txt" || true
# shellcheck disable=SC2016 # a jq program, which the shell must not expand
SRC='.packages[] | select(.name == $c and .version == $v) | .manifest_path'
cargo metadata -q --manifest-path "${M}" --format-version 1 >"${P}/metadata.json"
manifest=$(jq -r --arg c "${c}" --arg v "${v}" "${SRC}" "${P}/metadata.json")
dirname "${manifest}" >"${P}/srcdir.txt"
CD=$(rustup which clippy-driver)
# The target dir is shared, and cargo does not replay the cached warnings of a registry crate.
# A crate that an earlier probe built would show no warning, so the crate is cleaned first.
# (The first run did not do this, and gave os_pipe and winnow 0 blocks.)
cargo clean -q --manifest-path "${M}" --target "${TRIPLE}" --target-dir "${T}" -p "${c}@${v}"
# purba's environment denies warnings (build.warnings), and every forced lint is a warning.
# cargo's own "build-finished" line in check.json says whether the build passed.
CARGO_BUILD_WARNINGS=warn RUSTC_WRAPPER=${CD} \
  RUSTFLAGS="--force-warn unsafe_code --force-warn clippy::undocumented_unsafe_blocks" \
  cargo check -q --manifest-path "${M}" --target "${TRIPLE}" --target-dir "${T}" \
  --message-format json >"${P}/check.json" 2>"${P}/check.err" || true
jq -e 'select(.reason=="build-finished") | .success' "${P}/check.json" >/dev/null ||
  echo "build failed: ${c} ${v}" >&2
