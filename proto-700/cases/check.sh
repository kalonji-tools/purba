#!/usr/bin/env bash
# mypy --strict, pyright and ty on each way, then on its lie: test_c1 with
# src's fixture swapped for `count`, which returns an int.
# usage: PY=<python3.12> ./check.sh <scratch-dir>
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
lie() { # way -> perl script on the file that names src's fixture
	case $1 in
	w06_pytest) echo 'fixtures.py|s/def src\(source: Database\) -> Tx\[Database\]:\n    return Tx\(source\)/def src(source: Database) -> int:\n    return 0/' ;;
	w08_*) echo 'test_c1_copy.py|s/\@using\(src=source,/\@using(src=count,/' ;;
	w09_*) echo 'test_c1_copy.py|s/= source,/= count,/' ;;
	w10_*) echo 'test_c1_copy.py|s/use_any\(source\)/use_any(count)/' ;;
	w11_*) echo 'test_c1_copy.py|s/use\(tx, source\)/use(tx, count)/' ;;
	w12_*) echo 'test_c1_copy.py|s/From\(source\)/From(count)/' ;;
	*) echo '' ;;
	esac
}
checkers() {
	echo "--- $(uvx -q --python "$PY" mypy@latest --version) --strict"
	uvx -q --python "$PY" mypy@latest --strict --python-version 3.12 --no-incremental --cache-dir=/dev/null ./*.py 2>&1 || true
	echo "--- pyright $(uvx -q --from 'pyright[nodejs]@latest' pyright --version 2>&1 | tail -1)"
	uvx -q --from 'pyright[nodejs]@latest' pyright --pythonversion 3.12 --pythonpath "$PY" ./*.py 2>&1 | grep -E ' - (error|warning)|informations' | sed "s|$PWD/||" || true
	echo "--- $(uvx -q ty@latest --version)"
	uvx -q ty@latest check --python "$PY" --python-version 3.12 --output-format concise ./*.py 2>&1 || true
}
for way in "$here"/w*/; do
	name=$(basename "$way")
	echo "### $name"
	(cd "$way" && checkers)
	spec=$(lie "$name")
	[[ -z $spec ]] && { echo "### $name lie: cannot be written"; continue; }
	IFS='|' read -r file script <<<"$spec"
	dir=$1/lie/$name
	rm -rf "$dir" && mkdir -p "$dir" && cp "$way"*.py "$dir/"
	perl -0pi -e "$script" "$dir/$file"
	[[ $file == test_c1_copy.py ]] && sed -i 's/^from subject import /from subject import count, /' "$dir/$file"
	cmp -s "$way$file" "$dir/$file" && { echo "substitution missed $name" >&2; exit 1; }
	echo "### $name lie"
	(cd "$dir" && checkers)
done
