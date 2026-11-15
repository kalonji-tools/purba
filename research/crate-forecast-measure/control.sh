#!/usr/bin/env bash
# control.sh OUT
# The control for the unsafe reading of probe.sh. Run it under `mise exec --` from a purba
# checkout. Three path crates, each a dependency of one probe:
#   ctl_none  no unsafe                                     -> expect level 1
#   ctl_doc   one block with `// SAFETY:` above it            -> expect level 2
#   ctl_bare  one block with no comment, one `unsafe fn`, one `unsafe impl` with no comment
#                                                           -> expect level 3, and the fn and
#                                                              the impl not counted as blocks
set -euo pipefail
out=$1
C=${out}/control
rm -rf "${C}"
mkdir -p "${C}"
mk() {
  mkdir -p "${C}/$1/src"
  printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2024"\n' "$1" >"${C}/$1/Cargo.toml"
  cat >"${C}/$1/src/lib.rs"
}
mk ctl_none <<'EOF'
pub fn f(x: u8) -> u8 { x + 1 }
EOF
mk ctl_doc <<'EOF'
pub fn f(x: &[u8]) -> u8 {
    // SAFETY: the caller passes a slice of length at least 1.
    unsafe { *x.get_unchecked(0) }
}
EOF
mk ctl_bare <<'EOF'
pub struct S(*const u8);
unsafe impl Send for S {}
/// # Safety
/// x is not empty.
pub unsafe fn g(x: &[u8]) -> u8 { x[0] }
pub fn f(x: &[u8]) -> u8 {
    unsafe { *x.get_unchecked(0) }
}
EOF
mkdir -p "${C}/probe/src"
: >"${C}/probe/src/lib.rs"
cat >"${C}/probe/Cargo.toml" <<'EOF'
[package]
name = "probe_control"
version = "0.0.0"
edition = "2024"

[dependencies]
ctl_none = { path = "../ctl_none" }
ctl_doc = { path = "../ctl_doc" }
ctl_bare = { path = "../ctl_bare" }

[workspace]
EOF
CD=$(rustup which clippy-driver)
RUSTC_WRAPPER=${CD} \
  RUSTFLAGS="--force-warn unsafe_code --force-warn clippy::undocumented_unsafe_blocks" \
  cargo check -q --manifest-path "${C}/probe/Cargo.toml" --target x86_64-unknown-linux-gnu \
  --target-dir "${C}/target" --message-format json >"${C}/check.json" 2>/dev/null || true
MSG='select(.reason=="compiler-message")
  | [(.package_id | sub(".*/control/"; "")), .message.code.code, .message.message] | @tsv'
jq -r "${MSG}" "${C}/check.json" | sort
