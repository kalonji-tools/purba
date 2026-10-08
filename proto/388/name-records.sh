#!/usr/bin/env bash
# PROTOTYPE: these checks are off because the script is never run outside the replay.
# shellcheck disable=SC2001,SC2012,SC2016,SC2312
# PROTOTYPE for #388 — throwaway, never merges.
#
# Names the records a pull request must satisfy, and why each is named.
#
#   name-records.sh <base> <head> [records-dir]
#
# The reach of a record is read off its `**Reach:**` line: each code span is a
# gitattributes pattern, and each link to CONTEXT.md is a glossary word. The
# records are the ones at <base>, plus every record the pull request adds or
# changes (read at <head>). records-dir, when given, replaces the records at
# <base>: the replay uses it to put a reach on a base that has none.
#
# Prints one line per named record: <slug> TAB <reasons, comma-separated>.
set -u # no -e: a grep that finds nothing is not a failure here

base=$1 head=$2 dir=${3:-}
scratch=$(mktemp -d)
trap 'rm -rf "${scratch}"' EXIT

# The words a pull request always writes: every pull request has commit
# messages, a body, and an issue it closes.
always=${ALWAYS-'commit-message pull-request issue'}

mkdir -p "${scratch}/records"
if [[ -n "${dir}" ]]; then
  cp "${dir}"/*.md "${scratch}/records/"
else
  git ls-tree --name-only "${base}" docs/decisions/ | grep -v -e '/\.template' -e '/README' |
    while read -r f; do git show "${base}:${f}" >"${scratch}/records/${f##*/}"; done
fi

git diff --name-only --no-renames "${base}...${head}" >"${scratch}/changed"
# A record the pull request adds or changes is named, and its reach comes from <head>.
grep '^docs/decisions/[^/.][^/]*\.md$' "${scratch}/changed" | while read -r f; do
  slug=${f##*/}
  slug=${slug%.md}
  echo "${slug}	changed"
  git show "${head}:${f}" >"${scratch}/records/${slug}.md" 2>/dev/null || true
done >"${scratch}/hits"

# One attribute per record: reach-<slug>.
: >"${scratch}/attributes"
for r in "${scratch}"/records/*.md; do
  slug=$(basename "${r}" .md)
  line=$(grep -m1 '^\*\*Reach:\*\*' "${r}" || true)
  [[ -n "${line}" ]] || {
    echo "${slug}	NO-REACH"
    continue
  }
  grep -o '`[^`]*`' <<<"${line}" | tr -d '`' | while read -r pattern; do
    echo "${pattern} reach-${slug}" >>"${scratch}/attributes"
  done
  grep -o 'CONTEXT\.md#[a-z-]*' <<<"${line}" | sed 's/.*#//' | while read -r word; do
    if [[ " ${always} " == *" ${word} "* ]]; then
      echo "${slug}	word:${word}"
    else
      echo "${slug}	word-unmatched:${word}"
    fi
  done
done >>"${scratch}/hits"

git init --quiet --template= "${scratch}/empty"
GIT_ATTR_NOSYSTEM=1 git -C "${scratch}/empty" -c core.attributesFile="${scratch}/attributes" \
  check-attr --all --stdin <"${scratch}/changed" |
  awk -F': ' '$2 ~ /^reach-/ && $3 == "set" { sub(/^reach-/, "", $2); print $2 "\treach:" $1 }' >>"${scratch}/hits"

# The links in files stay and count: a changed file that names a record at <base>.
while read -r f; do
  # LINKS=headers counts a link only from a file outside docs/decisions/.
  if [[ "${LINKS:-all}" == none ]]; then continue; fi
  if [[ "${LINKS:-all}" == headers && "${f}" == docs/decisions/* ]]; then continue; fi
  git show "${base}:${f}" 2>/dev/null | grep -o '[a-z0-9-]*\.md' | sort -u |
    while read -r link; do
      if [[ -e "${scratch}/records/${link}" ]]; then echo "${link%.md}	link:${f}"; fi
    done
done <"${scratch}/changed" >>"${scratch}/hits"

grep -v -e 'NO-REACH' -e 'word-unmatched' "${scratch}/hits" | sort -u |
  awk -F'\t' '{ r[$1] = r[$1] ? r[$1] "," $2 : $2 } END { for (s in r) print s "\t" r[s] }' | sort
grep -e 'NO-REACH' -e 'word-unmatched' "${scratch}/hits" | sort -u | sed 's/^/# /' >&2 || true
