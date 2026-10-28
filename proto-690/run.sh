#!/usr/bin/env bash
# Runs every way on each Python, as written and under `from __future__ import annotations`.
# usage: PY312=<python3.12> PY314=<python3.14> ./run.sh <scratch-dir>
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
scratch=$1
mkdir -p "$scratch/future"
for f in "$here"/ways/*.py; do
	# every module's docstring is its first line
	sed '1a from __future__ import annotations' "$f" >"$scratch/future/$(basename "$f")"
done
for py in "$PY312" "$PY314"; do
	for variant in ways future; do
		dir=$here/ways
		[[ $variant == future ]] && dir=$scratch/future
		echo "## $("$py" -c 'import platform; print(platform.python_version())') $variant"
		for way in one_value by_name newtype subclass annotated type_alias factory pair holder holder_via_builder; do
			"$py" -I -B "$here/run.py" "$dir" "$way" 2>&1 | cut -c1-170
		done
	done
done
