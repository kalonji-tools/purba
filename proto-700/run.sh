#!/usr/bin/env bash
# The probe on each Python, as written and under `from __future__ import annotations`.
# usage: PY312=<python3.12> PY314=<python3.14> ./run.sh <scratch-dir>
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
future=$1/future
rm -rf "$future"
mkdir -p "$future"
for f in "$here"/ways/*.py; do
	sed '1a from __future__ import annotations' "$f" >"$future/$(basename "$f")"
done
for py in "$PY312" "$PY314"; do
	"$py" -I -B "$here/probe.py" "$here/ways"
	"$py" -I -B "$here/probe.py" "$future"
done
