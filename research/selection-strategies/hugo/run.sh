#!/usr/bin/env bash
# Hugo finds pages by location (content/), and a marker file decides what the
# other files of a folder are: index.md makes them resources of one page,
# _index.md makes the folder a section. draft: true in front matter hides a page.
# usage: nix-shell -p hugo --run "bash run.sh <empty dir>"
set -u
D=$1
cd "$D" || exit 1
echo "== hugo version"
hugo version | cut -d' ' -f1-2

mkdir -p site/content/blog/bundle site/content/blog/loose site/layouts/_default site/notes
cd site || exit 1
printf 'baseURL = "https://example.org/"\ntitle = "t"\ndisableKinds = ["taxonomy", "term", "RSS", "sitemap", "robotsTXT", "404"]\n' >hugo.toml
printf '{{ .Content }}\n' >layouts/_default/single.html
printf '{{ range .Pages }}{{ .Title }} {{ end }}\n' >layouts/_default/list.html
printf -- '---\ntitle: blog\n---\n' >content/blog/_index.md
printf -- '---\ntitle: post\n---\nbody\n' >content/blog/post.md
printf -- '---\ntitle: draft\ndraft: true\n---\nbody\n' >content/blog/draft.md
printf 'no front matter\n' >content/blog/plain.md
printf -- '---\ntitle: underscore\n---\nbody\n' >content/blog/_private.md
# a leaf bundle: index.md marks the page; helper.md beside it becomes a resource
printf -- '---\ntitle: bundle\n---\nbody\n' >content/blog/bundle/index.md
printf -- '---\ntitle: helper\n---\nbody\n' >content/blog/bundle/helper.md
# a folder with no index file: each .md in it is a page of the section
printf -- '---\ntitle: loose\n---\nbody\n' >content/blog/loose/one.md
# a page outside content/
printf -- '---\ntitle: notes\n---\nbody\n' >notes/n.md

echo "== hugo build: each page it writes"
hugo --quiet 2>&1 | grep -v '^$'
find public -name '*.html' | sort
echo "== the list page of the section blog"
cat public/blog/index.html
