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

top=$(git rev-parse --show-toplevel 2>/dev/null) || {
  printf 'the manifests can only be found inside a git repository.\n' >&2
  exit 2
}
cd "${top}"

tools=$(MISE_OFFLINE=1 mise ls --current --json) || {
  printf 'mise.toml cannot be read.\n' >&2
  exit 2
}
crates=$(cargo metadata --no-deps --format-version 1 --manifest-path Cargo.toml) || {
  printf 'Cargo.toml cannot be read.\n' >&2
  exit 2
}
scripts_list=$(git ls-files 'scripts/*.rs' '.github/*.rs') || {
  printf 'the tracked cargo scripts cannot be listed.\n' >&2
  exit 2
}
scripts='[]'
while read -r script; do
  [[ -n "${script}" ]] || continue
  manifest=$(cargo metadata -Zscript --no-deps --format-version 1 --manifest-path "${script}") || {
    printf '%s cannot be read.\n' "${script}" >&2
    exit 2
  }
  scripts=$(jq -c --arg file "${script}" --argjson manifest "${manifest}" \
    '. + [$manifest.packages[].dependencies[] | {file: $file, as: .name, name: .name}]' \
    <<<"${scripts}")
done <<<"${scripts_list}"
mise config get -f pyproject.toml >/dev/null || {
  printf 'pyproject.toml cannot be read.\n' >&2
  exit 2
}

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
  | "  \(.[0].name): " + (map("\(.file) as \(.as)") | unique | join(", "))
') || {
  printf 'the manifests cannot be compared.\n' >&2
  exit 2
}

declared=()
for key in build-system.requires project.dependencies project.optional-dependencies \
  dependency-groups; do
  value=$(mise config get -f pyproject.toml "${key}" 2>/dev/null) || continue
  [[ "${value}" == "[]" ]] || declared+=("  ${key}")
done

broken=0

[[ -z "${conflicts}" ]] || {
  report "A package is refused when two manifests declare it, because the two declarations can \
drift apart." \
    "${conflicts}"
  broken=1
}

[[ ${#declared[@]} -eq 0 ]] || {
  detail=$(printf '%s\n' "${declared[@]}")
  report "A package in pyproject.toml is refused until purba chooses its Python manager. The \
change that declares the first one gives this check a reader for it." "${detail}"
  broken=1
}

exit "${broken}"
