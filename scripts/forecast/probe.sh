#!/usr/bin/env bash
# probe.sh OUT CRATE VERSION
# Run it from a purba checkout under `mise exec --`, so that cargo, clippy-driver and
# PYO3_PYTHON are purba's pinned ones.
#
# Makes one probe crate that depends on CRATE = "=VERSION" with default features, then:
#   1. blast: `cargo tree -e normal,build --target x86_64-unknown-linux-gnu`, distinct
#      "name version", without the probe and without the crate itself.
#   2. unsafe: `cargo check` with clippy-driver as the wrapper and two forced lints,
#      `unsafe_code` and `clippy::undocumented_unsafe_blocks`. A forced lint is not capped on a
#      dependency, so the warnings cover what the probe compiles.
#   3. src: the directory of the crate's published source, for the `#[test]` count of tests.
# Writes OUT/probe/CRATE/{tree.txt,set.txt,metadata.json,check.json,srcdir.txt}. The target dir
# is shared.
#
# This builds the crate, so it runs the build scripts and proc macros of the crate and of what
# it brings. Run it only in a scratch directory.
set -euo pipefail
out=$1
crate=$2
version=$3
probe=${out}/probe/${crate}
target=${out}/target
triple=x86_64-unknown-linux-gnu
rm -rf "${probe}"
mkdir -p "${probe}/src" # cargo makes ${target} itself, so that `cargo clean` accepts it
: >"${probe}/src/lib.rs"
cat >"${probe}/Cargo.toml" <<EOF
[package]
name = "probe_${crate//-/_}"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
${crate} = "=${version}"

[workspace]
EOF
manifest=${probe}/Cargo.toml
cargo generate-lockfile -q --manifest-path "${manifest}"
cargo tree -q --manifest-path "${manifest}" -e normal,build --prefix none --target "${triple}" \
  >"${probe}/tree.txt"
awk '{print $1" "$2}' "${probe}/tree.txt" | sort -u | grep -v '^probe_' |
  grep -vxF "${crate} v${version}" >"${probe}/set.txt" || true
# shellcheck disable=SC2016 # a jq program, which the shell must not expand
SRC='.packages[] | select(.name == $c and .version == $v) | .manifest_path'
cargo metadata -q --manifest-path "${manifest}" --format-version 1 >"${probe}/metadata.json"
published=$(jq -r --arg c "${crate}" --arg v "${version}" "${SRC}" "${probe}/metadata.json")
[[ -z "${published}" ]] || dirname "${published}" >"${probe}/srcdir.txt"
driver=$(rustup which clippy-driver)
# The target dir is shared, and cargo does not replay the cached warnings of a registry crate.
# A crate that an earlier probe built would show no warning, so the crate is cleaned first.
cargo clean -q --manifest-path "${manifest}" --target "${triple}" --target-dir "${target}" \
  -p "${crate}@${version}"
# purba's environment denies warnings (build.warnings), and every forced lint is a warning.
# cargo's own "build-finished" line in check.json says whether the build passed.
CARGO_BUILD_WARNINGS=warn RUSTC_WRAPPER=${driver} \
  RUSTFLAGS="--force-warn unsafe_code --force-warn clippy::undocumented_unsafe_blocks" \
  cargo check -q --manifest-path "${manifest}" --target "${triple}" --target-dir "${target}" \
  --message-format json >"${probe}/check.json" 2>/dev/null || true
jq -e 'select(.reason=="build-finished") | .success' "${probe}/check.json" >/dev/null ||
  echo "build failed: ${crate} ${version}" >&2
