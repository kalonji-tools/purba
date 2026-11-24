#!/usr/bin/env bash
# Write CHECKERS.txt.gz for each version under benchmark/, from ty and pyright run
# on a copy in $1/check/<version>, with that copy's path shown as `.`.
# $1 = the eval folder; it holds the venv at $1/.venv.
set -euo pipefail
E="$1"
K="$(cd "$(dirname "$0")/../benchmark" && pwd)"
for v in pytest oxitest purba; do
  C="$E/check/$v"
  rm -rf "$C"
  mkdir -p "$C"
  (cd "$K/$v" && tar --exclude=__pycache__ --exclude=CHECKERS.txt.gz -cf - .) | (cd "$C" && tar -xf -)
  printf '{\n  "venvPath": "%s",\n  "venv": ".venv",\n  "include": ["tests"]\n}\n' "$E" >"$C/pyrightconfig.json"
  {
    echo "\$ ty check --python <venv>/bin/python tests"
    ty --version
    (cd "$C" && ty check --python "$E/.venv/bin/python" tests 2>&1) || true
    echo
    echo "\$ nix-shell -p pyright --run pyright"
    (cd "$C" && nix-shell -p pyright --run 'pyright --version; pyright' 2>&1) || true
    # gzip: the typos hook rewrites a word of pyright's summary line.
  } | sed -e "s|$C|.|g" -e "s|$E/.venv|<venv>|g" | gzip -n -9 >"$K/$v/CHECKERS.txt.gz"
done
