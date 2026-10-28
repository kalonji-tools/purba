#!/usr/bin/env bash
# Regenerates the cases, then probes each way on each Python,
# as written and under `from __future__ import annotations`.
# usage: PY312=<python3.12> PY314=<python3.14> ./run.sh <scratch-dir>
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
"$PY314" -I -B "$here/gen.py"
rm -rf "$1/future"
for way in "$here"/w*/; do
	name=$(basename "$way")
	mkdir -p "$1/future/$name"
	for f in "$way"*.py; do
		sed '1i from __future__ import annotations' "$f" >"$1/future/$name/$(basename "$f")"
	done
done
for py in "$PY312" "$PY314"; do
	for variant in written future; do
		echo "## Python $("$py" -c 'import platform; print(platform.python_version())'), $variant"
		root=$here
		[[ $variant == future ]] && root=$1/future
		for way in "$root"/w*/; do
			"$py" -I -B "$here/probe.py" "$way"
		done
	done
done
