#!/usr/bin/env bash
# Probe: which configuration ruff takes for each file, and the order of its levels.
# Usage: ruff.sh <ruff-binary> <empty-work-dir>
# Each module trips three rules: F401 (unused import), E741 (name `l`), E711 (== None).
set -u
RUFF=$1
W=$2
mkdir -p "$W"
cd "$W" || exit 1
export HOME="$W/home" XDG_CONFIG_HOME="$W/home/.config"
mkdir -p "$XDG_CONFIG_HOME/ruff"
unset RUFF_OUTPUT_FORMAT RUFF_NO_CACHE RUFF_CACHE_DIR
export RUFF_NO_CACHE=true
mod='import os
l = 1
x = l == None
'
codes() {
  "$RUFF" check --output-format concise "$@" 2>&1 | grep -oE '(F401|E741|E711)|warning.*|error.*' | sort | uniq | tr '\n' ' '
  echo
}
run() {
  echo "\$ (cd ${PWD#$W/}) ruff check $*"
  echo "  -> $(codes "$@")"
}

mkdir -p proj/sub
printf '%s' "$mod" >proj/m.py
printf '%s' "$mod" >proj/sub/m.py

echo "## 1. a user file only: \$XDG_CONFIG_HOME/ruff/ruff.toml selects E741"
printf '[lint]\nselect = ["E741"]\n' >"$XDG_CONFIG_HOME/ruff/ruff.toml"
(cd proj && run m.py)
echo "## 2. plus a project pyproject.toml with no [tool.ruff]: is the user file still taken?"
printf '[project]\nname = "p"\nrequires-python = ">=3.12"\n' >proj/pyproject.toml
(cd proj && run m.py)
echo "## 3. plus a project ruff.toml that selects F401: the user file is dropped, not merged"
printf '[lint]\nselect = ["F401"]\n' >proj/ruff.toml
(cd proj && run m.py)
echo "## 4. a nested sub/ruff.toml that selects E711: the closest file wins for sub/m.py, no merge"
printf '[lint]\nselect = ["E711"]\n' >proj/sub/ruff.toml
(cd proj && run m.py sub/m.py)
echo "   the same from inside sub/ and from /"
(cd proj/sub && run m.py ../m.py)
(cd / && run "$W/proj/m.py" "$W/proj/sub/m.py")
echo "## 5. sub/ruff.toml with extend = \"../ruff.toml\" and extend-select: merge only when asked"
printf 'extend = "../ruff.toml"\n[lint]\nextend-select = ["E711"]\n' >proj/sub/ruff.toml
(cd proj && run sub/m.py)
printf '[lint]\nselect = ["E711"]\n' >proj/sub/ruff.toml
echo "## 6. the command line: --select replaces every file's list, --extend-select adds to each"
(cd proj && run m.py sub/m.py --select E741)
(cd proj && run m.py sub/m.py --extend-select E741)
(cd proj && run m.py sub/m.py --config 'lint.select = ["E741"]')
echo "## 7. --config <file> names one file for every module and drops the hierarchy"
(cd proj && run m.py sub/m.py --config ruff.toml)
echo "## 8. --isolated ignores every file"
(cd proj && run m.py sub/m.py --isolated)
echo "## 9. in one directory: .ruff.toml beats ruff.toml beats pyproject.toml"
printf '[tool.ruff.lint]\nselect = ["E711"]\n' >>proj/pyproject.toml
(cd proj && run m.py)
printf '[lint]\nselect = ["E741"]\n' >proj/.ruff.toml
(cd proj && run m.py)
rm proj/.ruff.toml proj/ruff.toml
(cd proj && run m.py)
echo "## 10. code: a noqa comment beats every file and the command line; --ignore-noqa beats the comment"
printf 'import os  # noqa: F401\nl = 1  # noqa\nx = l == None\n' >proj/n.py
(cd proj && run n.py --select F401,E741,E711)
(cd proj && run n.py --select F401,E741,E711 --ignore-noqa)
printf '# ruff: noqa: E711\nimport os\nl = 1\nx = l == None\n' >proj/n.py
(cd proj && run n.py --select F401,E741,E711)
echo "## 11. a stricter-wins check: is there one? --select ALL with a file's ignore"
printf '[tool.ruff.lint]\nignore = ["E741"]\n' >proj/pyproject.toml
(cd proj && run m.py --select F401,E741,E711)
(cd proj && run m.py --select F401,E741,E711 --extend-select E741)
