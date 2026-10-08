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

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/../../scripts/report.sh"

if [[ $# -ne 1 ]]; then
  cannot "usage: require-abi3.sh <directory>"
fi

floor=$(sed -n 's/^requires-python = ">=3\.\([0-9][0-9]*\)"$/\1/p' pyproject.toml)
if [[ -z "${floor}" ]]; then
  cannot "pyproject.toml names no requires-python of the form >=3.N"
fi
want="cp3${floor}-abi3"

shopt -s nullglob
wheels=("$1"/*.whl)
if [[ ${#wheels[@]} -eq 0 ]]; then
  cannot "$1 holds no wheel"
fi

# No field of a wheel's name holds a dash, so `-cp3N-abi3-` can only be its
# Python and ABI tags.
# https://packaging.python.org/en/latest/specifications/binary-distribution-format/
wrong=()
for wheel in "${wheels[@]}"; do
  [[ "${wheel##*/}" == *"-${want}-"* ]] || wrong+=("${wheel##*/}")
done

if [[ ${#wrong[@]} -gt 0 ]]; then
  refuse "A wheel is uploaded only as ${want}, because one abi3 wheel loads on every \
CPython from the floor up that is not free-threaded, and requires-python names that floor to a \
resolver. These are not:" "${wrong[@]}"
  finish
fi

echo "every wheel in $1 is ${want}"
