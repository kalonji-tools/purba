#!/usr/bin/env bash
# Sass marks a helper, not a unit: a file whose name starts with `_` is a partial,
# and compiling a folder writes no CSS for it.
# usage: nix-shell -p dart-sass --run "bash run.sh <empty dir>"
set -u
D=$1
cd "$D" || exit 1
echo "== sass --version"
sass --version

mkdir -p in
printf '$brand: red;\n' >in/_colors.scss
printf '@use "colors";\nbody { color: colors.$brand; }\n' >in/site.scss
printf '$size: 1px;\n' >in/sizes.scss
echo "== 1. compile the folder: which CSS files it writes"
sass --no-source-map in:out 2>&1
find out -type f | sort

echo "== 2. a partial and a plain file of the same name"
printf '$brand: blue;\n' >in/colors.scss
sass --no-source-map in:out2 2>&1 | head -3
