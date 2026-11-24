#!/usr/bin/env bash
# Jekyll reads three markers: a name that starts with `_` hides a file, front
# matter marks a file to render, and a post's name must carry a date.
# usage: nix-shell -p jekyll --run "bash run.sh <empty dir>"
set -u
D=$1
cd "$D" || exit 1
echo "== jekyll --version"
jekyll --version

mkdir -p site/_posts site/_layouts site/notes
cd site || exit 1
printf 'title: t\n' >_config.yml
printf '{{ content }}\n' >_layouts/default.html
printf -- '---\nlayout: default\n---\n# page\n' >page.md
printf '# no front matter\n' >plain.md
printf -- '---\n---\n# hidden\n' >_hidden.md
printf -- '---\n---\n# note\n' >notes/n.md
printf -- '---\n---\n# post\n' >_posts/2026-01-02-dated.md
printf -- '---\n---\n# post\n' >_posts/undated.md

echo "== jekyll build --verbose: the lines that warn, skip or call a file invalid"
jekyll build --verbose 2>&1 | grep -iE 'warn|invalid|skip|undated|_hidden'
echo "(end of those lines)"
echo "== each file it writes"
find _site -type f | sort
