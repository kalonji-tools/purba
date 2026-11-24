#!/usr/bin/env bash
# Red-team the raters' load-bearing claims on fresh copies of rater1's folders.
# $1 = eval folder.
set -u
E="$1"
V="$E/.venv/bin"
S="$E/../rt"
rm -rf "$S"
mkdir -p "$S"
cp -r "$E/blind/rater1/ORIGINAL_pytest" "$S/py"
cp -r "$E/blind/rater1/PORT_R" "$S/ox"
cp -r "$E/blind/rater1/PORT_T" "$S/pb"
t() {
  echo "\$ $*"
  timeout 120 "$@" >"$S/out.txt" 2>&1
  local c=$?
  grep -vE '^\s*$|═|^\s+│|┌|└' "$S/out.txt" | tail -n 8
  echo "exit=$c"
  echo
}

echo "##### C9 oxitest: tset_ name under strict"
sed -i 's/^def test_progressive_format/def tset_progressive_format/' "$S/ox/tests/test_add_option_format.py"
cd "$S/ox" && t "$V/oxitest" --color=never
echo "##### C9 purba: tset_ name"
sed -i 's/^def test_progressive_format/def tset_progressive_format/' "$S/pb/tests/test_add_option_format.py"
cd "$S/pb" && t "$V/purba" tests

echo "##### C8 purba: tag with no import purba"
cp -r "$E/blind/rater1/PORT_T" "$S/pb2"
sed -i 's/^import purba$//' "$S/pb2/tests/test_internals.py"
cd "$S/pb2" && t "$V/purba" 'tag(smoke)'
t "$V/purba" tests

echo "##### C10 pytest: --lf drops a fixed parametrized case"
sed -i 's/("b", "Nope", "Nope\\n")/("b", "Nope", "Nope?\\n")/' "$S/py/tests/test_add_option_format.py"
cd "$S/py" && t "$V/pytest" -q
t "$V/pytest" -q --lf
sed -i 's/("b", "Nope", "Nope?\\n")/("b", "Nope", "Nope\\n")/' "$S/py/tests/test_add_option_format.py"
t "$V/pytest" -q --lf
t "$V/pytest" -q --lf

echo "##### C10 purba: second-path when logger.add stops raising"
cp -r "$E/blind/rater1/PORT_T" "$S/pb3"
grep -n 'def test_invalid_format_builtin' -A4 "$S/pb3/tests/test_add_option_format.py"
