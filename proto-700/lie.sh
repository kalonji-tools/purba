#!/usr/bin/env bash
# Each way whose line names a provider, with src's provider swapped for `count`,
# which returns an int. Can a type checker see that the line lies?
# usage: ./lie.sh <scratch-dir>
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
out=$1/lie
rm -rf "$out"
mkdir -p "$out"
cp "$here"/ways/{subject,sketch}.py "$out/"
# module|perl substitution on the whole file
variants=(
	'w06_parameter_name|s/def source\(source_db: Database\) -> Tx\[Database\]:\n    return Tx\(source_db\)/def source(source_db: Database) -> int:\n    return 0/'
	'w07_unannotated_name|s/def source_tx\(source_db: Database\) -> Tx\[Database\]:\n    return Tx\(source_db\)/def source_tx(source_db: Database) -> int:\n    return 0/'
	'w08_decorator_map|s/\@using\(src=source,/\@using(src=count,/'
	'w09_default_bare|s/= source,/= count,/'
	'w10_default_untyped_marker|s/use_any\(source\)/use_any(count)/'
	'w11_default_typed_marker|s/use\(tx, source\)/use(tx, count)/'
	'w12_annotated_provider|s/From\(source\)/From(count)/'
	'w13_alias_hiding_provider|s/From\(source\)/From(count)/'
	'w14_container_attribute|s/Provide\[Stocks.source\]/Provide[count]/'
)
for row in "${variants[@]}"; do
	IFS='|' read -r module script <<<"$row"
	perl -0pe "$script" "$here/ways/$module.py" | sed 's/^from subject import /from subject import count, /' >"$out/$module.py"
	if diff -q <(sed 's/^from subject import /from subject import count, /' "$here/ways/$module.py") "$out/$module.py" >/dev/null; then
		echo "substitution missed $module" >&2
		exit 1
	fi
done
ls "$out"
