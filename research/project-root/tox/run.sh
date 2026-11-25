#!/usr/bin/env bash
# tox's tox_root: the folder of the first settings file up from the working
# folder; with none, the nearest pyproject.toml, or else the working folder.
# usage: bash run.sh <tox 4.67.16> <empty dir>
set -u
TOX=$1
D=$2
export HOME=$D/home XDG_CONFIG_HOME=$D/home/.config
unset TOX_CONFIG_FILE
mkdir -p "$HOME" "$D/p/a"
echo "== tox --version"
"$TOX" --version 2>&1 | grep -v '^ROOT:' | awk '{print $1}'

# show <cwd> <args...>: tox's warning, if any, and tox_root
show() {
  local cwd=$1
  shift
  echo "-- cwd ${cwd#"$D"/}: tox config --core -k tox_root $*"
  (cd "$cwd" && "$TOX" config --core -k tox_root "$@" 2>&1 | grep -E '^(ROOT:|tox_root)')
}

echo "== 1. no file anywhere"
show "$D/p/a"
echo "== 2. p/.git: git does not mark tox_root"
mkdir -p "$D/p/.git"
show "$D/p/a"
echo "== 3. p/pyproject.toml with no tox table: the fallback takes its folder"
printf '[project]\nname = "p"\n' >"$D/p/pyproject.toml"
show "$D/p/a"
echo "== 4. p/tox.ini"
printf '[tox]\nenv_list = py\n' >"$D/p/tox.ini"
show "$D/p/a"
echo "== 5. --root p/a overrides it"
show "$D/p/a" --root .
echo "== 6. -c p/tox.ini from the folder above p"
show "$D" -c p/tox.ini
