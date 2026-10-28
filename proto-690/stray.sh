#!/usr/bin/env bash
# The stray builder: `seeded` names the plain type (`db: Database`) instead of
# the test's source role. Which database does it seed?
# usage: PY=<python> ./stray.sh <scratch-dir>
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
out=$1/stray
mkdir -p "$out"
cp "$here"/ways/common.py "$out"/
for way in by_name newtype subclass annotated type_alias pair holder holder_via_builder; do
	sed -E \
		-e 's/def seeded\((src: [A-Za-z]+|dbs: tuple\[Database, Database\]|bench: CopyBench)\)/def seeded(db: Database)/' \
		-e 's/^    (src|dbs\[0\]|bench\.src)\.insert/    db.insert/' \
		"$here/ways/$way.py" >"$out/$way.py"
	grep -q 'def seeded(db: Database)' "$out/$way.py" || { echo "sed missed $way" >&2; exit 1; }
done
for way in by_name newtype subclass annotated type_alias pair holder holder_via_builder; do
	"$PY" -I -B "$here/run.py" "$out" "$way" 2>&1 | cut -c1-170
done
