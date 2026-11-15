#!/usr/bin/env bash
# Build three blind rater folders for the loguru file, laid out as round 5 was
# (prototypes/967-python-form-4/benchmark/ratings/build_blind_loguru.sh on proto/980).
# $1 = the eval folder to create; $2 = the folder of the form port under loguru-format/ (default form).
# The ports are read from this benchmark folder.
set -euo pipefail
E="$1"
PORT="${2:-form}"
B="$E/blind"
H="$(cd "$(dirname "$0")/.." && pwd)" # benchmark/
F="$(cd "${H}/.." && pwd)"            # the form: FORM.md, run.py, cases.py
mkdir -p "$E"
[ -x "$E/.venv/bin/python" ] || { echo "make $E/.venv first" >&2; exit 1; }
scrub() { sed -E -e 's/ ?A throwaway form for a blind( rating)?\.?//g' -e 's/ ?Never merge\.//g' -e 's/purba-form/form/g' "$@"; }
build() { # $1 rater, $2 letter for oxitest, $3 letter for the form
  local R="$B/rater$1"
  mkdir -p "$R/ORIGINAL_pytest" "$R/PORT_$2" "$R/PORT_$3/tests" "$R/_review"
  cp "${H}"/loguru-format/pytest/{test_add_option_format.py,conftest.py,LICENSE.loguru,OUTPUT.txt,CHECKERS.txt} "$R/ORIGINAL_pytest/"
  printf '{"venvPath": "%s", "venv": ".venv"}\n' "$E" >"$R/ORIGINAL_pytest/pyrightconfig.json"
  printf 'Run from this folder:\n\n    %s/.venv/bin/pytest -v -p no:cacheprovider\n' "$E" >"$R/ORIGINAL_pytest/RUN.md"
  (cd "${H}/loguru-format/oxitest" && tar --exclude=__pycache__ --exclude=.oxitest_cache --exclude=LICENSE.loguru -cf - .) | (cd "$R/PORT_$2" && tar -xf -)
  sed -i -E "s|^cd eval2/oxitest$|cd $R/PORT_$2|; s|\.\./\.venv/|$E/.venv/|g" "$R/PORT_$2/RUN.md"
  printf '{\n  "venvPath": "%s",\n  "venv": ".venv",\n  "include": ["tests"]\n}\n' "$E" >"$R/PORT_$2/pyrightconfig.json"
  local P="$R/PORT_$3"
  scrub "${F}/run.py" >"$P/run.py"
  scrub "${F}/cases.py" >"$P/cases.py"
  cp "${F}/FORM.md" "$P/FORM.md"
  scrub "${H}/loguru-format/${PORT}/test_add_option_format.py" >"$P/tests/test_add_option_format.py"
  cp "${H}/loguru-format/${PORT}/LICENSE.loguru" "$P/tests/"
  for f in NOTES.md CHECKERS.txt; do scrub "${H}/loguru-format/${PORT}/$f" >"$P/$f"; done
  printf '{"venvPath": "%s", "venv": ".venv", "extraPaths": [".", "tests"], "include": ["tests"]}\n' "$E" >"$P/pyrightconfig.json"
  printf 'Run from this folder:\n\n    %s/.venv/bin/python run.py tests\n\nRules of the form: FORM.md\n' "$E" >"$P/RUN.md"
  (cd "$P" && "$E/.venv/bin/python" run.py tests >OUTPUT.txt 2>&1 || true)
}
build 1 M K
build 2 K M
build 3 M K
