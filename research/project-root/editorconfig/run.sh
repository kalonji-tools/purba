#!/usr/bin/env bash
# EditorConfig reads every .editorconfig from the file's folder up, and stops
# at the first that says root = true, or at the root of the file system.
# usage: bash run.sh <editorconfig-core-py 0.17.1 CLI> <empty dir>
set -u
EC=$1
D=$2
mkdir -p "$D/p/a"
: >"$D/p/a/m.py"
echo "== editorconfig --version"
"$EC" --version

show() {
  echo "-- editorconfig <dir>/p/a/m.py"
  "$EC" "$D/p/a/m.py"
  echo "exit=$?"
}

echo "== 1. no .editorconfig anywhere: no property, no error"
show
echo "== 2. <dir>/.editorconfig (root = true) and p/.editorconfig (no root): both apply"
printf 'root = true\n\n[*]\nindent_size = 8\n' >"$D/.editorconfig"
printf '[*]\nindent_style = tab\n' >"$D/p/.editorconfig"
show
echo "== 3. p/.editorconfig says root = true: the walk stops there"
printf 'root = true\n\n[*]\nindent_style = tab\n' >"$D/p/.editorconfig"
show

echo "== 4. the flags of the CLI"
"$EC" -h 2>&1 | grep -E -- "^-f"
