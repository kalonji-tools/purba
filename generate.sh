#!/usr/bin/env bash
# PROTOTYPE for purba #537: derives cli-lib-core/ and crate-per-part/ from one-crate/.
#
#   ./generate.sh .
set -euo pipefail
cd "$1" # the prototype worktree
main_toml=$(git show origin/main:Cargo.toml)
parts=(parsing selection fixtures assertion execution dispatch reporting command)
core_parts=(parsing selection fixtures assertion execution dispatch reporting)

# one-crate: main's own manifest, unchanged.
printf '%s\n' "${main_toml}" >one-crate/Cargo.toml

# A workspace root: main's manifest, with its lint tables moved to the workspace.
root_toml() { # $1 members (TOML array body), $2 the root's path dependency line
  printf '%s\n' "${main_toml}" |
    sed -e 's/^\[lints\.\(rust\|rustdoc\|clippy\|cargo\)\]/[workspace.lints.\1]/' \
      -e "s|^pyo3 = .*|pyo3 = { workspace = true }\n$2|"
  cat <<EOF

[lints]
workspace = true

[workspace]
members = [$1]
resolver = "3"

[workspace.package]
version = "0.0.0"
edition = "2024"
license = "MIT"
publish = false

[workspace.dependencies]
pyo3 = { version = "0.29.2", features = ["abi3-py312"] }
EOF
}

member_toml() { # $1 name, rest: dependency lines
  local name=$1
  shift
  printf '[package]\nname = "%s"\nversion.workspace = true\nedition.workspace = true\nlicense.workspace = true\npublish.workspace = true\n\n[dependencies]\n' "${name}"
  printf '%s\n' "$@"
  printf '\n[lints]\nworkspace = true\n'
}

uses() { sed -n 's/^use crate::\([a-z]*\);$/\1/p' "one-crate/src/$1.rs"; }
needs_pyo3() { grep -q '^use pyo3' "one-crate/src/$1.rs"; }
root_lib() { # $1 the crate command's run lives in
  sed -e '/^pub mod /d' -e "s/command::run/$1::run/" \
    -e "s|layout ONE CRATE: every part is a module of the one crate|layout $2|" one-crate/src/lib.rs | cat -s
}

# crate-per-part
rm -rf crate-per-part && mkdir -p crate-per-part/src
members=$(printf '"src/%s", ' "${parts[@]}")
root_toml "${members%, }" 'purba-command = { path = "src/command" }' >crate-per-part/Cargo.toml
root_lib purba_command 'CRATE PER PART: the extension module; each part is a crate under src/' >crate-per-part/src/lib.rs
for part in "${parts[@]}"; do
  mkdir -p "crate-per-part/src/${part}/src"
  deps=()
  needs_pyo3 "${part}" && deps+=('pyo3 = { workspace = true }')
  while read -r used; do
    [[ -n "${used}" ]] && deps+=("purba-${used} = { path = \"../${used}\" }")
  done < <(uses "${part}")
  member_toml "purba-${part}" "${deps[@]}" >"crate-per-part/src/${part}/Cargo.toml"
  sed 's/^use crate::\([a-z]*\);$/use purba_\1 as \1;/' "one-crate/src/${part}.rs" \
    >"crate-per-part/src/${part}/src/lib.rs"
done

# cli-lib-core
rm -rf cli-lib-core && mkdir -p cli-lib-core/src/cli/src cli-lib-core/src/core/src
root_toml '"src/cli", "src/core"' 'purba-cli = { path = "src/cli" }' >cli-lib-core/Cargo.toml
root_lib purba_cli 'CLI/LIB/CORE: lib is the extension module, cli holds command, core holds every other part' >cli-lib-core/src/lib.rs
member_toml purba-cli 'purba-core = { path = "../core" }' >cli-lib-core/src/cli/Cargo.toml
sed 's/^use crate::\([a-z]*\);$/use purba_core::\1;/' one-crate/src/command.rs >cli-lib-core/src/cli/src/lib.rs
member_toml purba-core 'pyo3 = { workspace = true }' >cli-lib-core/src/core/Cargo.toml
{
  echo '//! PROTOTYPE core: every part but command, each a module.'
  printf 'pub mod %s;\n' $(printf '%s\n' "${core_parts[@]}" | sort)
} >cli-lib-core/src/core/src/lib.rs
for part in "${core_parts[@]}"; do cp "one-crate/src/${part}.rs" cli-lib-core/src/core/src/; done
for layout in cli-lib-core crate-per-part; do cp one-crate/pyproject.toml "${layout}/"; done
echo generated
