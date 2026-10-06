#!/usr/bin/env bash
# The packages that two of mise.toml, Cargo.toml and the manifest inside a cargo
# script declare, and any package pyproject.toml declares.
#
#   the decision:  docs/decisions/one-manifest-declares-each-package.md
#   the command:   mise run lint:manifests
#
#   check-manifests.sh
#
# Exits 1 when it refuses a package, and 2 when a manifest cannot be read.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"

top=$(git rev-parse --show-toplevel 2>/dev/null) ||
  cannot 'the manifests can only be found inside a git repository.'
cd "${top}"

tools=$(MISE_OFFLINE=1 mise ls --current --json) ||
  cannot 'mise.toml cannot be read.'
crates=$(cargo metadata --no-deps --format-version 1 --manifest-path Cargo.toml) ||
  cannot 'Cargo.toml cannot be read.'
scripts_list=$(git ls-files 'scripts/*.rs' '.github/*.rs') ||
  cannot 'the tracked cargo scripts cannot be listed.'
scripts='[]'
while read -r script; do
  [[ -n "${script}" ]] || continue
  manifest=$(cargo metadata -Zscript --no-deps --format-version 1 --manifest-path "${script}") ||
    cannot "${script} cannot be read."
  scripts=$(jq -c --arg file "${script}" --argjson manifest "${manifest}" \
    '. + [$manifest.packages[].dependencies[] | {file: $file, as: .name, name: .name}]' \
    <<<"${scripts}")
done <<<"${scripts_list}"
mise config get -f pyproject.toml >/dev/null ||
  cannot 'pyproject.toml cannot be read.'

# shellcheck disable=SC2016 # jq's own variables, not the shell's
conflicts=$(jq -nr \
  --arg here "${top}/mise.toml" \
  --argjson tools "${tools}" \
  --argjson crates "${crates}" \
  --argjson scripts "${scripts}" '
  [
    ($tools | to_entries[]
      | select(any(.value[]; .source.path == $here))
      | {file: "mise.toml", as: .key, name: (.key | sub("^[^:]*:"; "") | split("/") | last)}),
    ($crates.packages[].dependencies[]
      | {file: "Cargo.toml", as: .name, name: .name}),
    $scripts[]
  ]
  | map(.name |= (ascii_downcase | gsub("[-_.]+"; "-")))
  | group_by(.name)
  | map(select(map(.file) | unique | length > 1))
  | .[]
  | "\(.[0].name): " + (map("\(.file) as \(.as)") | unique | join(", "))
') ||
  cannot 'the manifests cannot be compared.'

declared=()
for key in build-system.requires project.dependencies project.optional-dependencies \
  dependency-groups; do
  value=$(mise config get -f pyproject.toml "${key}" 2>/dev/null) || continue
  [[ "${value}" == "[]" ]] || declared+=("${key}")
done

[[ -z "${conflicts}" ]] || refuse \
  "A package is refused when two manifests declare it, because the two declarations can \
drift apart." \
  "${conflicts}"

[[ ${#declared[@]} -eq 0 ]] || refuse \
  "A package in pyproject.toml is refused until purba chooses its Python manager. The \
change that declares the first one gives this check a reader for it." \
  "${declared[@]}"

finish
