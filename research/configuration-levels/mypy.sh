#!/usr/bin/env bash
# Probe: which file mypy reads, and the order of its levels.
# Usage: mypy.sh <venv-bin-dir> <empty-work-dir>
# The module m.py has one function with no annotation. --disallow-untyped-defs reports it.
set -u
BIN=$1
W=$2
mkdir -p "$W"
cd "$W" || exit 1
export HOME="$W/home" XDG_CONFIG_HOME="$W/home/.config"
mkdir -p "$XDG_CONFIG_HOME/mypy" repo/a/b
unset MYPYPATH MYPY_CACHE_DIR
printf 'def f(x):\n    return x\n' >repo/a/b/m.py
cp repo/a/b/m.py repo/m.py
run() {
  echo "\$ (cd ${PWD#$W/}) mypy $*"
  "$BIN/mypy" --no-incremental "$@" 2>&1 | grep -E 'error|Success|config' | sed 's/^.*\(error\|Success\)/\1/' | head -3
  echo
}
verbose_cfg() { "$BIN/mypy" --no-incremental -v "$@" 2>&1 | grep -E 'Config File' | head -1; }

echo "## 1. a user file only: \$XDG_CONFIG_HOME/mypy/config"
printf '[mypy]\ndisallow_untyped_defs = True\n' >"$XDG_CONFIG_HOME/mypy/config"
(cd repo/a/b && verbose_cfg m.py && run m.py)
echo "## 2. a project file higher up is found from the cwd; the user file is dropped, not merged"
printf '[mypy]\nwarn_unused_ignores = True\n' >repo/mypy.ini
(cd repo/a/b && verbose_cfg m.py && run m.py)
echo "## 3. a .git directory stops the walk below the project file"
mkdir -p repo/a/.git
(cd repo/a/b && verbose_cfg m.py && run m.py)
rmdir repo/a/.git
echo "## 4. the walk starts at the cwd, not at the file given"
mkdir -p elsewhere # run from / crashes mypy 2.4.1 with an INTERNAL ERROR, so use an empty dir
(cd elsewhere && verbose_cfg "$W/repo/a/b/m.py" && run "$W/repo/a/b/m.py")
echo "## 5. a pyproject.toml with no [tool.mypy] does not stop the walk"
printf '[project]\nname = "x"\n' >repo/a/pyproject.toml
(cd repo/a/b && verbose_cfg m.py)
rm repo/a/pyproject.toml
echo "## 6. the command line beats the [mypy] section; a per-module section beats the command line"
printf '[mypy]\ndisallow_untyped_defs = True\n' >repo/mypy.ini
(cd repo && run m.py)
(cd repo && run m.py --allow-untyped-defs)
printf '[mypy]\ndisallow_untyped_defs = False\n[mypy-m]\ndisallow_untyped_defs = True\n' >repo/mypy.ini
(cd repo && run m.py)
(cd repo && run m.py --allow-untyped-defs)
printf '[mypy]\ndisallow_untyped_defs = True\n[mypy-m]\ndisallow_untyped_defs = False\n' >repo/mypy.ini
(cd repo && run m.py --disallow-untyped-defs)
echo "## 7. code: an inline '# mypy:' comment beats the per-module section and the command line"
printf '# mypy: allow-untyped-defs\ndef f(x):\n    return x\n' >repo/m.py
printf '[mypy]\n[mypy-m]\ndisallow_untyped_defs = True\n' >repo/mypy.ini
(cd repo && run m.py --disallow-untyped-defs)
echo "   and a '# type: ignore' on the line"
printf 'def f(x):  # type: ignore[no-untyped-def]\n    return x\n' >repo/m.py
(cd repo && run m.py --disallow-untyped-defs)
echo "## 8. --strict on the command line, and a per-module False under it"
printf 'def f(x):\n    return x\n' >repo/m.py
printf '[mypy]\n[mypy-m]\ndisallow_untyped_defs = False\n' >repo/mypy.ini
(cd repo && run m.py --strict)
echo "## 9. --config-file names one file; MYPY_CONFIG_FILE_DIR is set by mypy, not read"
printf '[mypy]\ndisallow_untyped_defs = False\n' >other.ini
(cd repo && run m.py --config-file ../other.ini)
