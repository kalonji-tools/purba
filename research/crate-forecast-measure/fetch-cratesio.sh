#!/usr/bin/env bash
# fetch-cratesio.sh OUT CRATE...
# Fetches the crate record (with every version) and the owners of each crate from crates.io.
# Two requests per crate, 1.1 s apart: crates.io asks for at most 1 request per second.
# The User-Agent names the project, never a person.
set -euo pipefail
out=$1
shift
UA='purba-research (https://github.com/kalonji-tools/purba)'
mkdir -p "${out}"
for c in "$@"; do
  curl -sS -f -A "${UA}" "https://crates.io/api/v1/crates/${c}" -o "${out}/${c}.json"
  sleep 1.1
  curl -sS -f -A "${UA}" "https://crates.io/api/v1/crates/${c}/owners" -o "${out}/${c}.owners.json"
  sleep 1.1
done
