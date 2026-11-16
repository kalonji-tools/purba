#!/usr/bin/env bash
# Build three blind rater folders for the toolz file, laid out as the loguru rounds of
# prototypes/1015-form/benchmark/ratings/build_blind_loguru.sh, for the 1025 form.
# $1 = the eval folder to create; $2 = the venv with pytest 9.2.0 and oxitest 4.0.0.
# The ports are read from this benchmark folder.
set -euo pipefail
E="$1"
V="$2"
B="$E/blind"
H="$(cd "$(dirname "$0")/.." && pwd)" # benchmark/
F="$(cd "${H}/.." && pwd)"            # the form: FORM.md, run.py, cases.py
T="${H}/toolz-itertoolz"
mkdir -p "$E"
[ -x "$V/bin/python" ] || { echo "no venv at $V" >&2; exit 1; }
scrub() { sed -E -e 's/ ?A throwaway form for a blind( rating)?\.?//g' -e 's/ ?Never merge\.//g' -e 's/purba-form/form/g' -e 's|1025-form|form|g' "$@"; }
build() { # $1 rater, $2 letter for oxitest, $3 letter for the form
  local R="$B/rater$1"
  mkdir -p "$R/ORIGINAL_pytest" "$R/PORT_$2" "$R/PORT_$3/tests" "$R/_review"
  (cd "${T}" && tar --exclude=__pycache__ -cf - toolz-src LICENSE.toolz) | (cd "$R" && tar -xf -)
  # the original
  cp "${T}/pytest/test_itertoolz.py" "$R/ORIGINAL_pytest/"
  printf '{"venvPath": "%s", "venv": "%s", "extraPaths": ["../toolz-src"]}\n' "$(dirname "$V")" "$(basename "$V")" >"$R/ORIGINAL_pytest/pyrightconfig.json"
  printf '[environment]\nextra-paths = ["../toolz-src"]\n' >"$R/ORIGINAL_pytest/ty.toml"
  printf 'Run from this folder:\n\n    PYTHONPATH=../toolz-src %s/bin/pytest -v -p no:cacheprovider\n\nType checkers:\n\n    ty check --python %s/bin/python test_itertoolz.py\n    nix-shell -p pyright --run pyright\n' "$V" "$V" >"$R/ORIGINAL_pytest/RUN.md"
  (cd "$R/ORIGINAL_pytest" && PYTHONPATH=../toolz-src "$V/bin/pytest" -v -p no:cacheprovider >OUTPUT.txt 2>&1 || true)
  cp "${T}/pytest/CHECKERS.txt" "$R/ORIGINAL_pytest/"
  # oxitest
  local O="$R/PORT_$2"
  (cd "${T}/oxitest" && tar --exclude=__pycache__ --exclude=.oxitest_cache --exclude=results.json -cf - .) | (cd "$O" && tar -xf -)
  sed -i -E "s|^VENV=.*$|VENV=$V|" "$O/RUN.md"
  (cd "$O" && { PYTHONPATH=../toolz-src "$V/bin/oxitest" -vv --color=never 2>&1; echo "exit=$?"
    PYTHONPATH=../toolz-src "$V/bin/oxitest" --color=never --json "$R/_review/.ox.json" >/dev/null 2>&1 || true
    jq -r '.results.tests[] | "\(.status)  \(.name | sub(".*/tests/"; "tests/"))"' "$R/_review/.ox.json"; } >OUTPUT.txt; rm -f "$R/_review/.ox.json")
  # the form
  local P="$R/PORT_$3"
  scrub "${F}/run.py" >"$P/run.py"
  scrub "${F}/cases.py" >"$P/cases.py"
  cp "${F}/FORM.md" "$P/FORM.md"
  scrub "${T}/form/test_itertoolz.py" >"$P/tests/test_itertoolz.py"
  for f in NOTES.md CHECKERS.txt; do scrub "${T}/form/$f" >"$P/$f"; done
  printf '{"venvPath": "%s", "venv": "%s", "extraPaths": [".", "../toolz-src"], "include": ["tests"], "pythonVersion": "3.12"}\n' "$(dirname "$V")" "$(basename "$V")" >"$P/pyrightconfig.json"
  printf '[environment]\npython-version = "3.12"\nextra-paths = [".", "../toolz-src"]\n\n[src]\ninclude = ["tests"]\n' >"$P/ty.toml"
  printf 'Run from this folder:\n\n    PYTHONPATH=../toolz-src %s/bin/python run.py tests\n\nType checkers:\n\n    ty check --python %s/bin/python\n    nix-shell -p pyright --run pyright\n\nRules of the form: FORM.md\n' "$V" "$V" >"$P/RUN.md"
  (cd "$P" && PYTHONPATH=../toolz-src "$V/bin/python" run.py tests >OUTPUT.txt 2>&1 || true)
}
build 1 M K
build 2 K M
build 3 M K
