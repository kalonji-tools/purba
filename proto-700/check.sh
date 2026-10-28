#!/usr/bin/env bash
# mypy --strict, pyright and ty on every way, then on each lie variant.
# usage: PY=<python3.12> ./check.sh <scratch-dir>   (run lie.sh first)
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
for set in ways lie; do
	dir=$here/ways
	[[ $set == lie ]] && dir=$1/lie
	cd "$dir"
	echo "### $set"
	echo "--- $(uvx -q --python "$PY" mypy@latest --version) --strict"
	uvx -q --python "$PY" mypy@latest --strict --python-version 3.12 --no-incremental --cache-dir=/dev/null ./*.py 2>&1 || true
	echo "--- pyright $(uvx -q --from 'pyright[nodejs]@latest' pyright --version 2>&1 | tail -1)"
	uvx -q --from 'pyright[nodejs]@latest' pyright --pythonversion 3.12 --pythonpath "$PY" ./*.py 2>&1 | grep -E 'error|warning|informations' || true
	echo "--- $(uvx -q ty@latest --version)"
	uvx -q ty@latest check --python "$PY" --python-version 3.12 --output-format concise ./*.py 2>&1 || true
done
