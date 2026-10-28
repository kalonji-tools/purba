#!/usr/bin/env bash
# mypy --strict, pyright and ty on the generic modules and each variant.
# usage: PY=<python3.12> ./check_generic.sh <scratch-dir>   (run generic.sh first)
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
scratch=$1/gcheck
rm -rf "$scratch"
mkdir -p "$scratch/generic" "$scratch/variants"
cp "$here"/subject.py "$here"/sketch.py "$here"/ways/{adapter,common,generic_derived,generic_per_role,generic_typevar}.py "$scratch/generic/"
cp "$here"/subject.py "$here"/sketch.py "$1"/variants/*.py "$scratch/variants/"
for set in generic variants; do
	cd "$scratch/$set"
	echo "### $set"
	echo "--- $(uvx -q --python "$PY" mypy@latest --version) --strict"
	uvx -q --python "$PY" mypy@latest --strict --python-version 3.12 --no-incremental ./*.py 2>&1 || true
	echo "--- pyright $(uvx -q --from 'pyright[nodejs]@latest' pyright --version 2>&1 | tail -1)"
	uvx -q --from 'pyright[nodejs]@latest' pyright --pythonversion 3.12 --pythonpath "$PY" ./*.py 2>&1 | grep -E 'error|warning|informations' || true
	echo "--- $(uvx -q ty@latest --version)"
	uvx -q ty@latest check --python "$PY" --python-version 3.12 --output-format concise ./*.py 2>&1 || true
done
