#!/usr/bin/env bash
# Red-team, second pass, in the raters' own shape. $1 = eval folder.
set -u
E="$1"
V="$E/.venv/bin"
S="$E/../rt2"
rm -rf "$S"
mkdir -p "$S"
cp -r "$E/blind/rater1/ORIGINAL_pytest" "$S/py"
cp -r "$E/blind/rater1/PORT_T" "$S/pb"
cp -r "$E/blind/rater1/PORT_T" "$S/pb4"
t() {
  echo "\$ $*"
  timeout 120 "$@" >"$S/out.txt" 2>&1
  local c=$?
  grep -E 'purba:|cases:|passed|failed|picks|deselected|rerun|FAILED|second-path|project error' "$S/out.txt" | grep -v 'private name' | tail -n 8
  echo "exit=$c"
  echo
}

echo "##### C9 purba: the name-near line"
sed -i 's/^def test_progressive_format/def tset_progressive_format/' "$S/pb4/tests/test_add_option_format.py"
cd "$S/pb4" && t "$V/purba" tests
grep -E 'name-near|cases:' "$S/out.txt"
echo

echo "##### C8 purba: a tag on test_progressive_format, in a file with no import purba"
F="$S/pb/tests/test_add_option_format.py"
sed -i 's/^def test_progressive_format/@purba.tag("smoke")\ndef test_progressive_format/' "$F"
grep -c 'import purba' "$F"
cd "$S/pb" && t "$V/purba" 'tag(smoke)'

echo "##### C10 pytest: two failures, fix one, --lf"
P="$S/py/tests/test_add_option_format.py"
sed -i -e 's/("b", "Nope", "Nope\\n")/("b", "Nope", "Nope?\\n")/' -e 's/("a", "Message: {message}", "Message: a\\n")/("a", "Message: {message}", "Message: A\\n")/' "$P"
cd "$S/py" && t "$V/pytest" -q
t "$V/pytest" -q --lf
sed -i 's/("b", "Nope", "Nope?\\n")/("b", "Nope", "Nope\\n")/' "$P"
t "$V/pytest" -q --lf

echo "##### C8 oxitest: a misspelled mark in the query"
cp -r "$E/blind/rater1/PORT_R" "$S/ox"
cd "$S/ox" && t "$V/oxitest" --color=never -E 'mark(smok)'
t "$V/oxitest" --color=never -E 'mark(xyz)'

echo "##### C10 purba: logger.add stops raising in test_invalid_format_builtin"
cp -r "$E/blind/rater1/PORT_T" "$S/pb3"
G="$S/pb3/tests/test_add_option_format.py"
sed -i 's/^    error = add_error(format, ValueError)$/    error = add_error("{message}", ValueError)/' "$G"
cd "$S/pb3" && t "$V/purba" tests/test_add_option_format.py::test_invalid_format_builtin
