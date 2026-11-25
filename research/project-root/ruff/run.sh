#!/usr/bin/env bash
# Ruff's project root: the folder of the nearest settings file, or else the
# working folder; and the path that a diagnostic prints.
# usage: bash run.sh <ruff 0.17.6> <empty dir>
set -u
RUFF=$1
D=$2
export HOME=$D/home XDG_CONFIG_HOME=$D/home/.config
mkdir -p "$HOME" "$D/p/a"
printf 'import os\n' >"$D/p/a/m.py"
echo "== ruff --version"
"$RUFF" --version

# show <cwd> <args...>: the project root of a/m.py, then one diagnostic line
show() {
  local cwd=$1
  shift
  echo "-- cwd ${cwd#"$D"/}: ruff check $*"
  (cd "$cwd" && "$RUFF" check --show-settings "$@" "$D/p/a/m.py" 2>&1 |
    grep -E '^file_resolver.project_root')
  (cd "$cwd" && "$RUFF" check --output-format concise --select F401 "$@" "$D/p/a/m.py" 2>&1 |
    grep F401)
}

echo "== 1. no settings file anywhere"
show "$D/p"
show "$D/p/a"

echo "== 2. p/.git and p/pyproject.toml with no [tool.ruff]: neither marks the root"
mkdir -p "$D/p/.git"
printf '[project]\nname = "p"\n' >"$D/p/pyproject.toml"
show "$D/p/a"

echo "== 3. p/pyproject.toml with [tool.ruff]"
printf '[tool.ruff]\n' >"$D/p/pyproject.toml"
show "$D/p"
show "$D/p/a"

echo "== 4. --config p/pyproject.toml, run from p/a: the root is the working folder"
show "$D/p/a" --config "$D/p/pyproject.toml"

echo "== 5. no project file, a user file in \$XDG_CONFIG_HOME/ruff/ruff.toml"
rm "$D/p/pyproject.toml"
mkdir -p "$XDG_CONFIG_HOME/ruff"
printf '\n' >"$XDG_CONFIG_HOME/ruff/ruff.toml"
show "$D/p/a"
