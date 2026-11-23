#!/usr/bin/env bash
# Probe: cargo-nextest's command line, NEXTEST_* variables, per-test overrides and profiles.
# Usage: nextest.sh <empty-work-dir>
# The test `flaky` fails on its first try and passes on a retry, so "retries" decides its outcome.
set -u
W=$1
mkdir -p "$W"
cd "$W" || exit 1
export XDG_CONFIG_HOME="$W/xdg" CARGO_TARGET_DIR="$W/target" FLAKY_DIR="$W/flaky"
mkdir -p "$XDG_CONFIG_HOME" "$FLAKY_DIR"
for v in $(env | grep -o '^NEXTEST_[A-Z_]*'); do unset "$v"; done
cargo init -q --lib --name np --vcs none . 2>&1
cat >src/lib.rs <<'EOF'
#[test]
fn flaky() {
    let marker = std::path::Path::new(&std::env::var("FLAKY_DIR").unwrap()).join("seen");
    if !marker.exists() {
        std::fs::write(&marker, b"1").unwrap();
        panic!("first try fails");
    }
}
EOF
mkdir -p .config
cat >.config/nextest.toml <<'EOF'
[profile.default]
retries = 0

[[profile.default.overrides]]
filter = "test(flaky)"
retries = 1

[profile.ci]
retries = 0
EOF
cargo build -q --tests 2>&1 | tail -2
run() {
  rm -f "$FLAKY_DIR/seen"
  echo "\$ (cd ${PWD#$W/}) ${NENV:-}cargo nextest run $*"
  env ${NENV:-} cargo nextest run --no-fail-fast --color never "$@" 2>&1 | grep -E 'FLAKY|TRY|FAIL \[|Summary|warning: in user config' | sed "s/ \[ *[0-9.]*s\]//; s#$W/##g" | head -5
  echo
}

echo "## 1. the default profile's per-test override (retries = 1) beats the profile (retries = 0)"
run
echo "## 2. NEXTEST_RETRIES=0 beats the override; --retries 1 beats NEXTEST_RETRIES"
NENV="NEXTEST_RETRIES=0 " run
NENV="NEXTEST_RETRIES=0 " run --retries 1
echo "## 3. --profile ci sets retries = 0, but the override in the default profile still applies"
run --profile ci
NENV="NEXTEST_PROFILE=ci " run
echo "## 4. the file is read at the workspace root, also when nextest runs from a subdirectory"
(cd src && run)
echo "## 5. a .config/nextest.toml under the cwd that is not the workspace root is not read"
mkdir -p src/.config && printf '[profile.default]\nretries = 0\n[[profile.default.overrides]]\nfilter = "test(flaky)"\nretries = 0\n' >src/.config/nextest.toml
(cd src && run)
rm -r src/.config
echo "## 6. a user config cannot set retries: nextest refuses or ignores the key"
mkdir -p "$XDG_CONFIG_HOME/nextest"
printf '[profile.default]\nretries = 0\n' >"$XDG_CONFIG_HOME/nextest/config.toml"
run
rm -r "$XDG_CONFIG_HOME/nextest"
