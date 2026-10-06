#!/usr/bin/env bash
# Time the same check as a binary crate and as a cargo script, each from a cold
# build directory, then one run of each over the records. Run corpus.sh first.
#
# Needs the nightly that `mise.toml` names, for `-Zscript`.
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
records=("${here}"/corpus/records/*.md)
lock=${work}/lock/Cargo.lock
mkdir -p "${work}/lock"

seconds() { # command...
  local start end tenths
  start=$(date +%s%N)
  "$@" >/dev/null 2>&1
  end=$(date +%s%N)
  tenths=$(((end - start) / 100000000))
  printf '%d.%d' "$((tenths / 10))" "$((tenths % 10))"
}

crate_cold() {
  cargo build --release --manifest-path "${here}/libwalk/Cargo.toml" --target-dir "${work}/crate"
  "${work}/crate/release/libwalk" np "${records[@]}"
}
script_release=(cargo -Zscript --config "resolver.lockfile-path=\"${lock}\"" run --release
  --manifest-path "${here}/script/libwalk.rs" --target-dir "${work}/release" -- np "${records[@]}")
script_debug=(cargo -Zscript run --manifest-path "${here}/script/libwalk.rs"
  --target-dir "${work}/debug" -- np "${records[@]}")

printf '| form | cold build and first run | each later run |\n|---|---|---|\n'
cold=$(seconds crate_cold)
run=$(seconds "${work}/crate/release/libwalk" np "${records[@]}")
printf '| a binary crate, release | %s s | %s s |\n' "${cold}" "${run}"
cold=$(seconds "${script_release[@]}")
run=$(seconds "${script_release[@]}")
printf '| a cargo script, release | %s s | %s s |\n' "${cold}" "${run}"
cold=$(seconds "${script_debug[@]}")
run=$(seconds "${script_debug[@]}")
printf '| a cargo script, debug | %s s | %s s |\n' "${cold}" "${run}"
rm -rf "${work}"
