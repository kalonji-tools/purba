#!/usr/bin/env bash
# uv's project root: the nearest pyproject.toml up from the working folder.
# Its workspace root: a pyproject.toml above that holds [tool.uv.workspace]
# and lists the project. A project command refuses with no pyproject.toml;
# `uv run` runs with no project.
# usage: bash run.sh <python 3.13.13> <empty dir>
set -u
PY=$1
D=$2
export HOME=$D/home XDG_CONFIG_HOME=$D/home/.config UV_CACHE_DIR=$D/cache
export UV_PYTHON=$PY UV_PYTHON_DOWNLOADS=never UV_OFFLINE=1 UV_NO_CONFIG=1
unset UV_PROJECT UV_WORKING_DIRECTORY VIRTUAL_ENV
mkdir -p "$HOME" "$D/p/a" "$D/w/pkgs/m/src"
echo "== uv --version"
uv --version

# show <cwd> <args...>
show() {
  local cwd=$1
  shift
  echo "-- cwd ${cwd#"$D"/}: uv $*"
  (
    cd "$cwd" && uv "$@" 2>&1 | grep -v -E '^warning: .*preview'
    echo "exit=${PIPESTATUS[0]}"
  )
}

echo "== 1. no pyproject.toml anywhere"
show "$D/p/a" workspace dir
show "$D/p/a" run --no-project python -I -c 'print("ran")'
show "$D/p/a" run python -I -c 'print("ran")'
echo "== 2. p/.git only"
mkdir -p "$D/p/.git"
show "$D/p/a" workspace dir
echo "== 3. p/pyproject.toml with no [project] table"
printf '[tool.ruff]\n' >"$D/p/pyproject.toml"
show "$D/p/a" workspace dir
echo "== 4. p/pyproject.toml with [project]"
printf '[project]\nname = "p"\nversion = "0"\nrequires-python = ">=3.13"\n' >"$D/p/pyproject.toml"
show "$D/p/a" workspace dir
echo "== 5. a workspace: w lists pkgs/m"
printf '[project]\nname = "w"\nversion = "0"\nrequires-python = ">=3.13"\n\n[tool.uv.workspace]\nmembers = ["pkgs/*"]\n' >"$D/w/pyproject.toml"
printf '[project]\nname = "m"\nversion = "0"\nrequires-python = ">=3.13"\n' >"$D/w/pkgs/m/pyproject.toml"
show "$D/w/pkgs/m/src" workspace dir
echo "== 6. --project and --directory, from p/a"
show "$D/p/a" --project "$D/w/pkgs/m" workspace dir
show "$D/p/a" --directory "$D/w" workspace dir
