#!/usr/bin/env bash
# Runs mypy --strict, pyright and ty on every way, and on the stray variants.
# usage: PY=<python3.12> ./check.sh <scratch-dir>   (run stray.sh first)
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
scratch=$1/check
rm -rf "$scratch"
mkdir -p "$scratch/ways" "$scratch/stray"
cp "$here"/subject.py "$here"/sketch.py "$here"/ways/*.py "$scratch/ways/"
cp "$here"/subject.py "$here"/sketch.py "$1"/stray/*.py "$scratch/stray/"
for set in ways stray; do
	cd "$scratch/$set"
	echo "### $set"
	echo "--- $(uvx -q --python "$PY" mypy@latest --version) --strict"
	uvx -q --python "$PY" mypy@latest --strict --python-version 3.12 --no-incremental ./*.py 2>&1 || true
	echo "--- pyright $(uvx -q --from 'pyright[nodejs]@latest' pyright --version 2>&1 | tail -1)"
	uvx -q --from 'pyright[nodejs]@latest' pyright --pythonversion 3.12 --pythonpath "$PY" ./*.py 2>&1 | tail -n 3 || true
	echo "--- $(uvx -q ty@latest --version)"
	uvx -q ty@latest check --python "$PY" --python-version 3.12 ./*.py 2>&1 || true
done
