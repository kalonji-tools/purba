#!/usr/bin/env bash
# The generic adapter Tx[SourceDb]: each module on each Python, as written and
# under `from __future__ import annotations`, then each variant.
# usage: PY312=<python3.12> PY314=<python3.14> ./generic.sh <scratch-dir>
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
scratch=$1
mkdir -p "$scratch/gfuture" "$scratch/variants"
mods=(generic_derived generic_per_role generic_typevar)
for f in "${mods[@]}" adapter common; do
	sed '1a from __future__ import annotations' "$here/ways/$f.py" >"$scratch/gfuture/$f.py"
done
for py in "$PY312" "$PY314"; do
	for variant in ways gfuture; do
		dir=$here/ways
		[[ $variant == gfuture ]] && dir=$scratch/gfuture
		echo "## $("$py" -c 'import platform; print(platform.python_version())') $variant"
		for way in "${mods[@]}"; do
			"$py" -I -B "$here/run_generic.py" "$dir" "$way" 2>&1 | cut -c1-210
		done
	done
done

# Variants: each changes one line of a module. name|module|sed script
variants=(
	'plain_db|generic_derived|s/def seeded\(src: Tx\[SourceDb\]\)/def seeded(src: Database)/; s/    src\.db\.insert/    src.insert/'
	'plain_db|generic_per_role|s/def seeded\(src: Tx\[SourceDb\]\)/def seeded(src: Database)/; s/    src\.db\.insert/    src.insert/'
	'tx_of_plain_db|generic_derived|s/def seeded\(src: Tx\[SourceDb\]\)/def seeded(src: Tx[Database])/'
	'resource_direct|generic_derived|s/def seeded\(src: Tx\[SourceDb\]\)/def seeded(src: SourceDb)/; s/    src\.db\.insert/    src.insert/'
	'builder_twice|generic_derived|s/def seeded\(src: Tx\[SourceDb\]\)/def seeded(src: SourceDb, again: SourceDb)/; s/    src\.db\.insert/    src.insert/'
	'test_twice|generic_derived|s/def test_copy\(src: Tx\[SourceDb\], dst: Tx\[TargetDb\]/def test_copy(src: Tx[SourceDb], dst: Tx[SourceDb]/'
	'test_names_resource|generic_derived|s/def test_copy\(src: Tx\[SourceDb\],/def test_copy(src: Tx[SourceDb], raw: SourceDb,/'
	'not_a_database|generic_derived|s/def test_copy\(src: Tx\[SourceDb\],/def test_copy(src: Tx[SourceDb], other: Tx[Seeded],/'
)
cp "$here/ways/adapter.py" "$here/ways/common.py" "$scratch/variants/"
echo "## variants"
for row in "${variants[@]}"; do
	IFS='|' read -r label module script <<<"$row"
	sed -E "$script" "$here/ways/$module.py" >"$scratch/variants/${label}__$module.py"
	if cmp -s "$here/ways/$module.py" "$scratch/variants/${label}__$module.py"; then
		echo "sed missed $label" >&2
		exit 1
	fi
	"$PY312" -I -B "$here/run_generic.py" "$scratch/variants" "${label}__$module" 2>&1 |
		grep -E ' copy +rollback=on ' | cut -c1-230
	"$PY312" -I -B "$here/run_generic.py" "$scratch/variants" "${label}__$module" 2>&1 |
		grep -E ' bug +rollback=on ' | cut -c1-230
done
