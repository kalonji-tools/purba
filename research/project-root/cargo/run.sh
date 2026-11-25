#!/usr/bin/env bash
# cargo's package root: the nearest Cargo.toml up from the working folder.
# Its workspace root: the nearest Cargo.toml above that holds [workspace] and
# lists the package. Outside a package, cargo refuses. nextest names a test by
# its binary and its path in Rust, from any working folder.
# usage: bash run.sh <empty dir>
set -u
D=$1
export CARGO_TARGET_DIR=$D/target
mkdir -p "$D/w/crates/c/src" "$D/w/crates/c/tests" "$D/none"
echo "== cargo --version; cargo nextest --version"
cargo --version
cargo nextest --version 2>&1 | head -1
printf '[workspace]\nmembers = ["crates/c"]\nresolver = "3"\n' >"$D/w/Cargo.toml"
printf '[package]\nname = "c"\nversion = "0.1.0"\nedition = "2024"\n' >"$D/w/crates/c/Cargo.toml"
printf '#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() {}\n}\n' >"$D/w/crates/c/src/lib.rs"
printf '#[test]\nfn lends() {}\n' >"$D/w/crates/c/tests/it.rs"

# show <cwd> <args...>
show() {
  local cwd=$1
  shift
  echo "-- cwd ${cwd#"$D"/}: cargo $*"
  (
    cd "$cwd" && cargo "$@" 2>&1
    echo "exit=$?"
  )
}

echo "== 1. outside a package"
show "$D/none" locate-project --message-format plain
echo "== 2. from w/crates/c/src: the package, then the workspace"
show "$D/w/crates/c/src" locate-project --message-format plain
show "$D/w/crates/c/src" locate-project --workspace --message-format plain
echo "== 3. --manifest-path from outside"
show "$D/none" locate-project --workspace --message-format plain --manifest-path ../w/crates/c/Cargo.toml
echo "== 4. the ids of the tests, from w and from w/crates/c/src"
(cd "$D/w" && cargo nextest list --offline 2>/dev/null)
(cd "$D/w/crates/c/src" && cargo nextest list --offline 2>/dev/null)
