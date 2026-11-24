#!/usr/bin/env bash
# What Cargo does when an explicit list overrides its folder convention, and what
# rustc does with a mark on a unit that the unit's shape cannot carry.
# usage: bash run.sh <empty dir>
set -u
D=$1
cd "$D" || exit 1
echo "== cargo --version"
cargo --version

make_crate() {
  cargo new -q --lib --vcs none "$1"
  mkdir -p "$1/tests"
  echo '#[test] fn listed() {}' >"$1/tests/listed.rs"
  echo '#[test] fn unlisted() {}' >"$1/tests/unlisted.rs"
}

echo "== 1. autotests = false, and one [[test]]: which test targets run"
make_crate off
cat >>off/Cargo.toml <<'EOF'

[[test]]
name = "listed"
path = "tests/listed.rs"
EOF
# autotests is a key of [package].
sed -i 's/^\[package\]$/[package]\nautotests = false/' off/Cargo.toml
(cd off && cargo test --offline 2>&1 | grep -E 'warning|Running|^test ')

echo "== 2. autotests left at its default, and one [[test]]: which test targets run"
make_crate on
cat >>on/Cargo.toml <<'EOF'

[[test]]
name = "listed"
path = "tests/listed.rs"
EOF
(cd on && cargo test --offline 2>&1 | grep -E 'warning|Running|^test ')

echo "== 3. a [[test]] whose file does not exist"
make_crate missing
cat >>missing/Cargo.toml <<'EOF'

[[test]]
name = "gone"
path = "tests/gone.rs"
EOF
(cd missing && cargo test --offline 2>&1 | grep -E 'error|gone' | head -3)

echo "== 4. #[test] on a function that takes an argument"
make_crate args
echo '#[test] fn takes(n: u32) { assert_eq!(n, 1); }' >args/tests/listed.rs
(cd args && cargo test --offline 2>&1 | grep -E '^error' | head -2)

echo "== 5. a function with no #[test] in tests/: never a test, and a dead-code warning"
make_crate nomark
printf 'fn forgot() { assert_eq!(1, 1); }\n#[test] fn marked() {}\n' >nomark/tests/listed.rs
(cd nomark && cargo test --offline 2>&1 | grep -E 'warning: function|^test ' | head -4)
