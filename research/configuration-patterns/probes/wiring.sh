#!/usr/bin/env bash
# Runs purba's lint:wiring check (scripts/check-wiring.sh) on shapes of a
# configuration crate, in a scratch clone of purba main at 9e221a74.
# Usage: wiring.sh <a clone of purba at 9e221a74>
set -uo pipefail
cd "$1" || exit 2

line='- **configuration.** States facts about the project, or tightens a check, in command, selection, dispatch and reporting.'
named="${line} Its crate is \`purba-configuration\`."

reset() {
  git checkout -q -- .
  rm -rf crates/configuration
}

crate() { # $1: extra dependency lines
  mkdir -p crates/configuration/src
  printf '[package]\nname = "purba-configuration"\nversion = "0.0.0"\nedition.workspace = true\nlicense.workspace = true\nrepository.workspace = true\npublish.workspace = true\n\n[dependencies]\n%b\n[lints]\nworkspace = true\n' "$1" >crates/configuration/Cargo.toml
  printf '//! probe\n' >crates/configuration/src/lib.rs
}

name_crate() {
  local tmp
  tmp=$(mktemp)
  awk -v old="${line}" -v new="${named}" '$0 == old { print new; next } { print }' LANDSCAPE.md >"${tmp}"
  mv "${tmp}" LANDSCAPE.md
}

use_from() { # $1: part
  awk -v dep="purba-configuration = { path = \"../configuration\" }" '
    { print }
    $0 == "[dependencies]" { print dep }
  ' "crates/$1/Cargo.toml" >"crates/$1/Cargo.toml.new"
  mv "crates/$1/Cargo.toml.new" "crates/$1/Cargo.toml"
}

run() {
  echo "== $1"
  bash scripts/check-wiring.sh 2>&1 | sed 's/^/   /'
  echo "   exit=${PIPESTATUS[0]}"
}

reset
run "G0 main, unchanged"
reset
crate ""
run "G1 crate purba-configuration, line names no crate"
reset
crate ""
name_crate
run "G2 crate named by the line, uses nothing"
reset
crate 'purba-errors = { path = "../errors" }'
name_crate
run "G3 crate uses purba-errors"
reset
crate 'purba-selection = { path = "../selection" }'
name_crate
run "G4 crate uses purba-selection"
reset
crate ""
name_crate
use_from dispatch
run "G5 dispatch uses the crate"
reset
crate ""
name_crate
use_from execution
run "G6 execution uses the crate"
reset
