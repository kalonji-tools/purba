#!/usr/bin/env bash
# Rebuild the environment a commit started from an editor's button gets:
# git, mise and prek on PATH, and no cargo and no typos.
#
# It is the environment that separated the two candidate hook entries. See
# README.md in this directory.
set -euo pipefail

bin=$(mktemp -d)
for tool in git mise sh bash env; do
  ln -sf "$(command -v "$tool")" "$bin/$tool"
done
ln -sf "$(mise which prek)" "$bin/prek"

echo "PATH holds: $(ls "$bin" | tr '\n' ' ')" >&2
env -i HOME="$HOME" PATH="$bin" "$@"
