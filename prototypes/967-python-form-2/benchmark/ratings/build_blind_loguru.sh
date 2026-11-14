#!/usr/bin/env bash
# Build three blind rater folders for prototype 2, laid out as round 2's were (prototype 1's benchmark/ratings/build_blind_loguru.sh).
# $1 = the eval folder: .venv/, pytest_orig/, oxitest/, proto2/ (the port, by the porter).
set -euo pipefail
E="$1"; B="$E/blind"
scrub() { sed -E -e 's/ ?Prototype 2 \(#967\)\. Never merge\.//g' -e 's/\(#96[0-9][^)]*\)//g' "$@"; }
build() {  # $1 rater, $2 letter for oxitest, $3 letter for the prototype
  local R="$B/rater$1"; mkdir -p "$R/ORIGINAL_pytest" "$R/PORT_$2" "$R/PORT_$3/experiments" "$R/_review"
  cp "$E"/pytest_orig/{test_add_option_format.py,conftest.py,LICENSE.loguru,OUTPUT.txt,CHECKERS.txt} "$R/ORIGINAL_pytest/"
  printf '{"venvPath": "%s", "venv": ".venv"}\n' "$E" > "$R/ORIGINAL_pytest/pyrightconfig.json"
  printf 'Run from this folder:\n\n    %s/.venv/bin/pytest -v -p no:cacheprovider\n' "$E" > "$R/ORIGINAL_pytest/RUN.md"
  (cd "$E/oxitest" && tar --exclude=__pycache__ --exclude=.oxitest_cache --exclude=LICENSE.loguru -cf - .) | (cd "$R/PORT_$2" && tar -xf -)
  sed -i -E "s|^cd eval2/oxitest$|cd $R/PORT_$2|" "$R/PORT_$2/RUN.md"
  printf '{\n  "venvPath": "%s",\n  "venv": ".venv",\n  "include": ["tests"]\n}\n' "$E" > "$R/PORT_$2/pyrightconfig.json"
  local P="$R/PORT_$3"
  scrub "$E/proto2/run.py" > "$P/run.py"; scrub "$E/proto2/experiment.py" > "$P/experiment.py"; cp "$E/proto2/FORM.md" "$P/FORM.md"
  for f in "$E"/proto2/experiments/*.py; do scrub "$f" > "$P/experiments/$(basename "$f")"; done
  cp "$E/proto2/experiments/LICENSE.loguru" "$P/experiments/"
  for f in NOTES.md OUTPUT.txt CHECKERS.txt; do scrub "$E/proto2/$f" | sed "s|$E/proto2|.|g" > "$P/$f"; done
  printf '{"venvPath": "%s", "venv": ".venv", "extraPaths": [".", "experiments"], "include": ["experiments"]}\n' "$E" > "$P/pyrightconfig.json"
  printf 'Run from this folder:\n\n    %s/.venv/bin/python run.py experiments\n\nRules of the form: FORM.md\n' "$E" > "$P/RUN.md"
}
build 1 K M; build 2 M K; build 3 K M
