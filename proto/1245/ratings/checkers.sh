#!/usr/bin/env bash
# Write CHECKERS.txt.gz for each version named after $2 under benchmark/, from
# ty and pyright run on a copy in $1/check/<version>, with that copy's path
# shown as `.`. The words of purba's builds do not reach a checker, so one venv
# serves both.
# $1 = the eval folder; $2 = a venv that holds purba, pytest, oxitest and
# loguru; $3... = the versions.
set -euo pipefail
E="$1"
V="$2"
shift 2
K="$(cd "$(dirname "$0")/../benchmark" && pwd)"
for v in "$@"; do
  C="$E/check/$v"
  rm -rf "$C"
  mkdir -p "$C"
  (cd "$K/$v" && tar --exclude=__pycache__ --exclude=CHECKERS.txt.gz -cf - .) | (cd "$C" && tar -xf -)
  printf '{\n  "venvPath": "%s",\n  "venv": "%s",\n  "include": ["tests"]\n}\n' \
    "$(dirname "$V")" "$(basename "$V")" >"$C/pyrightconfig.json"
  {
    echo "\$ ty check --python <venv>/bin/python tests"
    ty --version
    (cd "$C" && ty check --python "$V/bin/python" tests 2>&1) || true
    echo
    echo "\$ nix-shell -p pyright --run pyright"
    (cd "$C" && nix-shell -p pyright --run 'pyright --version; pyright' 2>&1) || true
    # gzip: the typos hook rewrites a word of pyright's summary line.
  } | sed -e "s|$C|.|g" -e "s|$V|<venv>|g" | gzip -n -9 >"$K/$v/CHECKERS.txt.gz"
done
