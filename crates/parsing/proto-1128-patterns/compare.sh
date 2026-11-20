#!/usr/bin/env bash
# PROTOTYPE for purba issue 1128. Throwaway: this branch is never merged.
#
# Runs the ast-grep rules and an installed purba on each module given, and
# prints each (module, line, check) that one of them finds and the other
# does not. Each row is "file:line [check]".
#
# usage: compare.sh <purba> <module.py>...
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
purba=$1
shift
checks='not-one-comparison|computed-expected'
work=$(mktemp -d)
trap 'rm -r -- "${work}"' EXIT

for module in "$@"; do
  directory=$(dirname "${module}")
  name=$(basename "${module}")
  # purba reads one module, from its own directory, as the bats tests run it.
  (cd "${directory}" && timeout 30 "${purba}" "${name}" 2>&1 || true) |
    grep -oE "\[(${checks})\] line [0-9]+" |
    sed -E "s|^\[([a-z-]+)\] line ([0-9]+)$|${module}:\2 [\1]|" >>"${work}/purba" || true
  ast-grep scan --config "${here}/sgconfig.yml" --json=stream "${module}" |
    jq -r --arg m "${module}" '"\($m):\(.range.start.line + 1) [\(.ruleId)]"' >>"${work}/patterns"
done

sort -u "${work}/purba" >"${work}/purba.sorted"
sort -u "${work}/patterns" >"${work}/patterns.sorted"
echo "== both: $(comm -12 "${work}/purba.sorted" "${work}/patterns.sorted" | wc -l)"
comm -12 "${work}/purba.sorted" "${work}/patterns.sorted"
echo "== purba only: $(comm -23 "${work}/purba.sorted" "${work}/patterns.sorted" | wc -l)"
comm -23 "${work}/purba.sorted" "${work}/patterns.sorted"
echo "== patterns only: $(comm -13 "${work}/purba.sorted" "${work}/patterns.sorted" | wc -l)"
comm -13 "${work}/purba.sorted" "${work}/patterns.sorted"
