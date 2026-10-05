#!/usr/bin/env bash
# Refuse to upload a wheel that is not abi3 at the floor `requires-python` names.
#
#   the caller:    .github/workflows/publish.yml
#   the floor:     docs/decisions/purba-supports-a-python-with-two-years-left.md
#
#   require-abi3.sh <directory>
#
# Reads `requires-python` from pyproject.toml in the working directory.
#
# Exits 0 when every wheel carries the tags, 1 when one does not, and 2 when it
# cannot decide.
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: require-abi3.sh <directory>" >&2
  exit 2
fi

floor=$(sed -n 's/^requires-python = ">=3\.\([0-9][0-9]*\)"$/\1/p' pyproject.toml)
if [[ -z "${floor}" ]]; then
  echo "::error::pyproject.toml names no requires-python of the form >=3.N" >&2
  exit 2
fi
want="cp3${floor}-abi3"

shopt -s nullglob
wheels=("$1"/*.whl)
if [[ ${#wheels[@]} -eq 0 ]]; then
  echo "::error::$1 holds no wheel" >&2
  exit 2
fi

# No field of a wheel's name holds a dash, so `-cp3N-abi3-` can only be its
# Python and ABI tags.
# https://packaging.python.org/en/latest/specifications/binary-distribution-format/
wrong=()
for wheel in "${wheels[@]}"; do
  [[ "${wheel##*/}" == *"-${want}-"* ]] || wrong+=("${wheel##*/}")
done

if [[ ${#wrong[@]} -gt 0 ]]; then
  echo "::error::A wheel is uploaded only as ${want}, because one abi3 wheel loads on every \
CPython from the floor up that is not free-threaded, and requires-python names that floor to a \
resolver. These are not:" >&2
  printf '  %s\n' "${wrong[@]}" >&2
  exit 1
fi

echo "every wheel in $1 is ${want}"
