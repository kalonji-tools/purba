#!/usr/bin/env bash
# Times each post-pass transcript that files.txt lists, and prints one row for
# each kind of run and one for all of them.
#
#   run.sh [<transcripts>]
#
# <transcripts> defaults to ~/.claude/projects, which files.txt is relative to.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=${1:-${HOME}/.claude/projects}
out=$(mktemp)
trap 'rm -f "${out}"' EXIT
while IFS= read -r file; do
  jq -s --slurpfile names "${here}/names.json" -f "${here}/analyze3.jq" "${root}/${file}" |
    jq -c --arg file "${file}" '. + {file: $file}' >>"${out}"
done <"${here}/files.txt"
jq -s -r -L "${here}" -f "${here}/agg3.jq" "${out}"
