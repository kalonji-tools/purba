#!/usr/bin/env bash
# What cargo test and cargo-nextest pick as a test target and as a test,
# and what Cargo's target discovery makes of a helper beside a binary.
# usage: bash run.sh <empty dir>
set -u
D=$1
cd "$D" || exit 1

cargo new -q --lib --vcs none shop
cd shop || exit 1
mkdir -p tests/support
cat >src/lib.rs <<'EOF'
pub fn make() -> u32 { 1 }

#[cfg(test)]
mod tests {
    fn helper() -> u32 { super::make() }   // no #[test]: never a test

    #[test]
    fn lends() { assert_eq!(helper(), 1); } // a test name with no "test"
}
EOF
cat >tests/lend.rs <<'EOF'
mod support;

#[test]
fn lends_from_outside() { assert_eq!(support::one(), shop::make()); }
EOF
cat >tests/common.rs <<'EOF'
pub fn one() -> u32 { 1 }                   // a helper as a file of tests/
EOF
cat >tests/support/mod.rs <<'EOF'
pub fn one() -> u32 { 1 }                   // a helper in a folder of tests/
EOF

echo "== cargo --version"
cargo --version
echo "== cargo test (each target it runs)"
cargo test --offline 2>&1 | grep -E 'Running|running [0-9]+ test|^test '
echo "== cargo nextest list"
cargo nextest --version 2>&1 | head -1
cargo nextest list --offline 2>&1 | grep -v -E '^\s*(Compiling|Finished|Starting)'
cd ..

echo "== a helper beside a binary in src/bin"
cargo new -q --bin --vcs none tool
cd tool || exit 1
mkdir -p src/bin/other
echo 'fn main() { println!("{}", 1); }' >src/bin/extra.rs
echo 'pub fn help() -> u32 { 1 }' >src/bin/helper.rs
echo 'mod util; fn main() { println!("{}", util::two()); }' >src/bin/other/main.rs
echo 'pub fn two() -> u32 { 2 }' >src/bin/other/util.rs
cargo build --offline 2>&1 | grep -E 'error|bin/helper|E0601' | head -4
