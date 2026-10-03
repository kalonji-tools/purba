#!/usr/bin/env bash
# PROTOTYPE, never merged. What does each candidate cost `mise.lock`, and does
# purba's own manifest gate admit it?
#
# Each candidate is added alone to a scratch clone of this branch, then
# `mise lock` runs with purba's settings. A row reports the platforms the entry
# covers, how many carry a checksum, the download size for linux-x64, and the
# exit of `mise run lint:manifests`.
set -u
src=$(git rev-parse --show-toplevel) || exit 2
export GITHUB_TOKEN=${GITHUB_TOKEN:-$(gh auth token)}
export MISE_NODE_COMPILE=0

candidates=(
  'rumdl|"aqua:rvben/rumdl" = "0.2.78"'
  'mado|"github:akiomik/mado" = "0.3.2"'
  'dprint|dprint = "0.58.0"'
  'markdownlint-cli2|node = "24"\n"npm:markdownlint-cli2" = "0.23.3"'
  'pymarkdownlnt|uv = "0.12"\n"pipx:pymarkdownlnt" = "0.9.40"'
  'harper-ls|harper-ls = "2.12.0"'
)

printf '| candidate | adds to [tools] | platforms locked | with checksum | linux-x64 download | lint:manifests |\n'
printf '|---|---|---:|---:|---:|---|\n'
for c in "${candidates[@]}"; do
  name=${c%%|*}
  lines=${c#*|}
  work=$(mktemp -d)
  git clone -q "${src}" "${work}/purba"
  cd "${work}/purba" || exit 2
  # Insert before [settings], so the entry lands inside [tools].
  awk -v add="$(printf '%b' "${lines}")" '/^\[settings\]/ && !done {print add "\n"; done=1} {print}' \
    mise.toml >mise.toml.new && mv mise.toml.new mise.toml
  mise trust -q mise.toml
  before=$(grep -c '^\[\[tools\.' mise.lock)
  if ! mise lock >"${work}/lock.log" 2>&1; then
    printf '| %s | `%s` | lock failed: %s | | | |\n' "${name}" "${lines//\\n/; }" \
      "$(grep -m1 -i error "${work}/lock.log" | cut -c1-80)"
    mise trust -q --untrust mise.toml
    cd / && rm -rf "${work}"
    continue
  fi
  # The entries this candidate added, read back from the lock.
  added=$(git diff -U0 mise.lock | sed -nE 's/^\+\[\[tools\.("?[^]"]+"?)\]\]$/\1/p' | tr -d '"')
  platforms=0 sums=0 size='-'
  for t in ${added}; do
    p=$(grep -c "^\[tools\.\"\?${t}\"\?\.\"platforms\." mise.lock)
    s=$(awk -v t="${t}" 'index($0, "[tools." t ".\"platforms.") || index($0, "[tools.\"" t "\".\"platforms.") {inside=1; next}
      /^\[/ {inside=0} inside && /^checksum/ {n++} END {print n+0}' mise.lock)
    platforms=$((platforms + p)) sums=$((sums + s))
  done
  api=$(awk '/platforms.linux-x64"\]/ {inside=1; next} /^\[/ {inside=0} inside && /^url_api/ {print $3}' \
    <(git diff -U0 mise.lock | sed -n 's/^+//p') | tr -d '"' | head -1)
  [[ -n "${api}" ]] && size=$(gh api "${api#https://api.github.com/}" --jq '.size / 1048576 | floor | tostring + " MB"' 2>/dev/null || printf '?')
  gate=$(mise run lint:manifests >/dev/null 2>&1 && printf 'pass' || printf 'exit %s' "$?")
  printf '| %s | `%s` | %s | %s | %s | %s |\n' "${name}" "${lines//\\n/; }" "${platforms}" "${sums}" "${size}" "${gate}"
  printf '  added lock entries: %s (was %s tool entries)\n' "${added//$'\n'/ }" "${before}" >&2
  mise trust -q --untrust mise.toml
  cd / && rm -rf "${work}"
done
