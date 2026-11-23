#!/usr/bin/env bash
# Probe: cargo's nested .cargo/config.toml files, $CARGO_HOME, CARGO_* variables and --config.
# Usage: cargo.sh <empty-work-dir>
# `cargo config get` is unstable; RUSTC_BOOTSTRAP=1 lets the stable cargo run it.
set -u
W=$1
mkdir -p "$W"
cd "$W" || exit 1
export CARGO_HOME="$W/cargo-home" RUSTC_BOOTSTRAP=1
mkdir -p "$CARGO_HOME" top/.cargo top/mid/.cargo top/mid/leaf/.cargo
for v in $(env | grep -o '^CARGO_[A-Z_]*' | grep -v '^CARGO_HOME$'); do unset "$v"; done
get() {
  echo "\$ (cd ${PWD#$W/}) ${CENV:-}cargo config get $*"
  env ${CENV:-} cargo -Zunstable-options config get --show-origin "$@" 2>&1 | sed "s#$W/##g"
  echo
}

printf '[build]\njobs = 1\nrustflags = ["--cfg=home"]\n[term]\nverbose = true\n' >"$CARGO_HOME/config.toml"
printf '[build]\njobs = 2\nrustflags = ["--cfg=top"]\n' >top/.cargo/config.toml
printf '[build]\nrustflags = ["--cfg=mid"]\n' >top/mid/.cargo/config.toml
printf '[build]\njobs = 4\n' >top/mid/leaf/.cargo/config.toml

echo "## 1. every file from the cwd up to / is read, then \$CARGO_HOME; the deepest wins a scalar"
(cd top/mid/leaf && get build.jobs)
(cd top/mid && get build.jobs)
echo "## 2. an array joins every file, the deeper ones later"
(cd top/mid/leaf && get build.rustflags)
echo "## 3. a key only the user file sets is still taken"
(cd top/mid/leaf && get term.verbose)
echo "## 4. an environment variable beats every file; --config beats the variable"
(cd top/mid/leaf && CENV="CARGO_BUILD_JOBS=8 " get build.jobs)
(cd top/mid/leaf && CENV="CARGO_BUILD_JOBS=8 " get build.jobs --config build.jobs=16)
echo "## 5. arrays from the variable and from --config: added or replacing?"
(cd top/mid/leaf && CENV="CARGO_BUILD_RUSTFLAGS=--cfg=env " get build.rustflags)
(cd top/mid/leaf && get build.rustflags --config 'build.rustflags=["--cfg=cli"]')
echo "## 6. --config <file> adds a file on top of the discovered ones"
printf '[build]\njobs = 32\n' >named.toml
(cd top/mid/leaf && get build.jobs --config "$W/named.toml")
echo "## 7. the walk starts at the cwd, not at the manifest: --manifest-path from elsewhere"
mkdir -p elsewhere
(cd top/mid/leaf && cargo init -q --name leafpkg --vcs none . 2>&1)
(cd elsewhere && echo "\$ (cd elsewhere) cargo build -v --manifest-path top/mid/leaf/Cargo.toml" && cargo build -v --manifest-path "$W/top/mid/leaf/Cargo.toml" 2>&1 | grep -oE -- '--cfg=[a-z]+|-j ?[0-9]+|Running.*rustc' | sort -u | head -5)
(cd top/mid/leaf && echo "\$ (cd top/mid/leaf) cargo build -v" && cargo build -v 2>&1 | grep -oE -- '--cfg=[a-z]+' | sort -u | head -5)
