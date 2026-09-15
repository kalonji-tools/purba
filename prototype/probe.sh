#!/usr/bin/env bash
# Shared criterion probe. Both arms record the same manifest, so the two are
# comparable row by row. Absence is data, so nothing here may fail the job.
set -u
arm="$1"; os="$2"
say () { printf '%s\n' "$*" >> "$GITHUB_STEP_SUMMARY"; }

say "## ${arm} — ${os} (\`$(uname -m 2>/dev/null || echo unknown)\`)"
say ""
say '| criterion | value |'
say '|---|---|'
say "| resolved toolchain | \`$(rustc --version 2>&1 | tr -d '\r')\` |"
missing=""
for c in cargo-clippy rustfmt rust-analyzer; do
  command -v "$c" >/dev/null 2>&1 || missing="$missing $c"
done
src="$(rustc --print sysroot 2>/dev/null)/lib/rustlib/src/rust/library/std/src/lib.rs"
[ -f "$src" ] || missing="$missing rust-src"
if [ -z "$missing" ]; then
  say '| four components | **all present** |'
else
  say "| four components | ⚠️ **missing:**$missing |"
fi
say "| C compiler | \`$(command -v cc 2>/dev/null || command -v clang 2>/dev/null || command -v gcc 2>/dev/null || echo 'none on PATH')\` |"
say "| compiler actually used | ${COMPILER_USED:-unrecorded} |"
