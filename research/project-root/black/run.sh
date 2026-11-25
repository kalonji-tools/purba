#!/usr/bin/env bash
# Black's project root: .git, .hg or a pyproject.toml with [tool.black], in
# each folder from the common base of the arguments up; else the root of the
# file system.
# usage: bash run.sh <black 26.10.1> <empty dir>
set -u
BLACK=$1
D=$2
export HOME=$D/home XDG_CONFIG_HOME=$D/home/.config
mkdir -p "$HOME" "$D/p/a" "$D/q"
printf 'x = 1\n' >"$D/p/a/m.py"
printf 'y = 1\n' >"$D/q/n.py"
echo "== black --version"
"$BLACK" --version | head -1

# show <cwd> <args...>: the line that names the root
show() {
  local cwd=$1
  shift
  echo "-- cwd ${cwd#"$D"/}: black $*"
  (cd "$cwd" && "$BLACK" -v --check "$@" 2>&1 | grep -E '^Identified')
}

echo "== 1. no marker anywhere"
show "$D/p/a" m.py
echo "== 2. p/.git"
mkdir -p "$D/p/.git"
show "$D/p/a" m.py
echo "== 3. p/a/pyproject.toml with no [tool.black] is skipped"
printf '[project]\nname = "a"\n' >"$D/p/a/pyproject.toml"
show "$D/p/a" m.py
echo "== 4. p/a/pyproject.toml with [tool.black] is nearer than p/.git"
printf '[tool.black]\n' >"$D/p/a/pyproject.toml"
show "$D/p/a" m.py
echo "== 5. arguments in p/a and q: their common base is above p/.git"
rm "$D/p/a/pyproject.toml"
show "$D" p/a/m.py q/n.py
