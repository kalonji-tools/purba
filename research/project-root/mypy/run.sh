#!/usr/bin/env bash
# mypy has two roots. Its walk to a settings file stops at .git or .hg. The
# name of a module comes from the __init__.py files, or from the working
# folder under --explicit-package-bases.
# usage: bash run.sh <mypy 2.4.1> <empty dir>
set -u
MYPY=$1
D=$2
export HOME=$D/home XDG_CONFIG_HOME=$D/home/.config
unset MYPYPATH
mkdir -p "$HOME" "$D/outer/p/pkg"
printf '[mypy]\n' >"$D/outer/mypy.ini"
printf 'x: int = 1\n' >"$D/outer/p/pkg/mod.py"
echo "== mypy --version"
"$MYPY" --version

# show <cwd> <args...>: the settings file, then the module name of mod.py
show() {
  local cwd=$1
  shift
  echo "-- cwd ${cwd#"$D"/}: mypy $*"
  (cd "$cwd" && "$MYPY" -v --no-incremental --cache-dir=/dev/null "$@" 2>&1 |
    grep -E 'Config File:|Parsing .*mod\.py' | sed -E 's/^LOG: +//')
}

echo "== 1. the walk to outer/mypy.ini, with no .git"
show "$D/outer/p/pkg" mod.py
echo "== 2. outer/p/.git stops the walk"
mkdir -p "$D/outer/p/.git"
show "$D/outer/p/pkg" mod.py

echo "== 3. the module name with no __init__.py"
show "$D/outer/p" pkg/mod.py
show "$D/outer/p/pkg" mod.py
echo "== 4. the same, with --explicit-package-bases: the working folder is a base"
show "$D/outer/p" --explicit-package-bases pkg/mod.py
show "$D/outer/p/pkg" --explicit-package-bases mod.py
echo "== 5. pkg/__init__.py: the name no longer depends on the working folder"
: >"$D/outer/p/pkg/__init__.py"
show "$D/outer/p" pkg/mod.py
show "$D/outer/p/pkg" mod.py
