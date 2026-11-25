#!/usr/bin/env bash
# Clone each repository of repos.tsv, one commit deep, at its ref.
# Print each name, its ref and the commit that the ref gave, and stop when
# that commit is not the commit of the row.
# usage: bash clone.sh <repos.tsv> <empty dir>
set -u
LIST=$1
D=$2
tail -n +2 "$LIST" | while IFS=$'\t' read -r name url ref commit _group; do
  git init -q "$D/$name"
  git -C "$D/$name" fetch -q --depth 1 "$url" "$ref" || exit 1
  git -C "$D/$name" -c advice.detachedHead=false checkout -q FETCH_HEAD
  got=$(git -C "$D/$name" rev-parse HEAD)
  echo "$name $ref $got"
  [ "$got" = "$commit" ] || {
    echo "MISMATCH $name: want $commit"
    exit 1
  }
done
