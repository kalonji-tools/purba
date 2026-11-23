#!/usr/bin/env bash
# How ruff 0.17.6, uv 0.13.7, cargo 1.99.0 and cargo-nextest 0.9.149 answer a
# key that no value owns, and a value at a level that is closed to it.
# Usage: tools.sh <empty work directory>
# Needs uvx (it runs ruff@0.17.6), uv, cargo and cargo-nextest on PATH.
set -u
work=$1
cd "${work}" || exit 2
export UV_CACHE_DIR="${work}/uv-cache"
scrub() {
  sed -e "s|${PATH}|<PATH>|g" -e "s|${work}|<work>|g" -e "s|${HOME}|~|g" -e '/^\s*$/d' -e '/Nextest run ID/d'
}
show() {
  echo "\$ $*" | scrub
  "$@" 2>&1 | scrub
  echo "exit=${PIPESTATUS[0]}"
}

echo '== ruff: a key no option owns, in [tool.ruff]'
mkdir -p ruff/xdg && cd ruff || exit 2
printf '[tool.ruff]\nline-lnegth = 80\n' >pyproject.toml
echo 'x = 1' >a.py
show env HOME="${PWD}" XDG_CONFIG_HOME="${PWD}/xdg" uvx -q ruff@0.17.6 check a.py
cd "${work}" || exit 2

echo '== uv: the same in uv.toml, then in [tool.uv], then a project-only field in the user file'
mkdir -p uv/a uv/b uv/c uv/xdg/uv
uvenv=(env -i PATH="${PATH}" HOME="${work}/uv" XDG_CONFIG_HOME="${work}/uv/xdg" XDG_CONFIG_DIRS="${work}/uv/none" UV_CACHE_DIR="${UV_CACHE_DIR}")
printf 'index-urll = "https://x"\n' >uv/a/uv.toml
(cd uv/a && show "${uvenv[@]}" uv --offline cache dir)
printf '[project]\nname = "b"\nversion = "0"\n[tool.uv]\nindex-urll = "https://x"\n' >uv/b/pyproject.toml
(cd uv/b && show "${uvenv[@]}" uv --offline cache dir)
printf '[sources]\nfoo = { path = "x" }\n' >uv/xdg/uv/uv.toml
(cd uv/c && show "${uvenv[@]}" uv --offline cache dir)

echo '== cargo: a misspelled key in a known table and an unknown table; a token on the command line'
mkdir -p cargo/p/src cargo/p/.cargo cargo/p/tests cargo/home cargo/xdg/nextest
cd cargo/p || exit 2
printf '[package]\nname = "p"\nversion = "0.0.0"\nedition = "2021"\n' >Cargo.toml
echo 'fn main() {}' >src/main.rs
echo '#[test] fn t() {}' >tests/t.rs
printf '[build]\njbos = 4\n[bluid]\njobs = 4\n' >.cargo/config.toml
show env CARGO_HOME="${work}/cargo/home" cargo check --offline
show env CARGO_HOME="${work}/cargo/home" cargo check --offline --config 'registry.token="x"'
rm .cargo/config.toml

echo '== cargo-nextest: a misspelled key in the project file; retries in the user file'
mkdir -p .config
printf '[profile.default]\nretires = 2\n' >.config/nextest.toml
printf '[profile.default]\nretries = 2\n' >"${work}/cargo/xdg/nextest/config.toml"
show env RUSTUP_HOME="${RUSTUP_HOME:-${HOME}/.rustup}" CARGO_HOME="${work}/cargo/home" \
  XDG_CONFIG_HOME="${work}/cargo/xdg" HOME="${work}/cargo" cargo nextest run --offline
