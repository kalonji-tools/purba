#!/usr/bin/env bash
# Compare `lib-np` with and without a dictionary of purba's own, which gives
# `grown` its verb form. Run corpus.sh first. Prints each line that changes.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
lib=${here}/libwalk/target/release/libwalk
[[ -x ${lib} ]] || cargo build --release --manifest-path "${here}/libwalk/Cargo.toml"
for name in records wild; do
  dir=${here}/corpus/${name}
  diff <("${lib}" np "${dir}"/*.md) <("${lib}" np+d "${dir}"/*.md) |
    sed -n "s|^> ${dir}/|${name}: |p" || true
done
