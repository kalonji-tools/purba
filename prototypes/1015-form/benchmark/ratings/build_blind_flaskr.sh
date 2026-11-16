#!/usr/bin/env bash
# Build three blind rater folders for flaskr's test_blog.py: the original and two ports, each
# rater with its own shuffle of the letters K and M (KEY_flaskr.txt). The 1015 form has one port.
# $1 = the eval folder; it must hold a .venv with pytest, oxitest 4.0.0 and flask installed.
set -euo pipefail
E="$1"
B="$E/blind"
H="$(cd "$(dirname "$0")/.." && pwd)" # benchmark/
F="$(cd "${H}/.." && pwd)"            # the form: FORM.md, run.py, cases.py
S="${H}/flaskr-blog"
[ -x "$E/.venv/bin/python" ] || {
  echo "make $E/.venv first" >&2
  exit 1
}
scrub() {
  sed -E -e 's/ ?A throwaway form for a blind( rating)?\.?//g' -e 's/ ?Never merge\.//g' \
    -e 's/purba-form/form/g' -e "s|/home/[^ ]*/benchmark/flaskr-blog/[a-z-]+/|./|g" \
    -e 's|benchmark/flaskr-blog/[a-z-]+|.|g' -e 's|\.\./flaskr-src|../APP|g' -e 's|flaskr-src|APP|g' \
    -e 's|\.\./\.\./\.venv|<venv>|g' -e 's|\.\./\.\./\.\.|.|g' -e 's|1015-form|form|g' "$@"
}
form_port() { # $1 source folder, $2 target folder
  local P="$2"
  mkdir -p "$P/tests"
  scrub "${F}/run.py" >"$P/run.py"
  scrub "${F}/cases.py" >"$P/cases.py"
  cp "${F}/FORM.md" "$P/FORM.md"
  for f in "$1"/*.py; do scrub "$f" >"$P/tests/$(basename "$f")"; done
  cp "$1/data.sql" "$P/tests/"
  for f in NOTES.md CHECKERS.txt OUTPUT_mutant-login.txt; do
    [ -f "$1/$f" ] && scrub "$1/$f" >"$P/$f"
  done
  printf '{"venvPath": "%s", "venv": ".venv", "extraPaths": [".", "tests", "../APP"], "include": ["tests"]}\n' "$E" >"$P/pyrightconfig.json"
  printf '[environment]\nextra-paths = [".", "tests", "../APP"]\n' >"$P/ty.toml"
  printf 'Run from this folder:\n\n    PYTHONPATH=../APP %s/.venv/bin/python run.py tests\n\nRules of the form: FORM.md\n' "$E" >"$P/RUN.md"
  (cd "$P" && PYTHONPATH=../APP "$E/.venv/bin/python" run.py tests >OUTPUT.txt 2>&1 || true)
}
oxi_port() { # $1 target folder
  local P="$1"
  mkdir -p "$P/tests"
  cp "${S}/oxitest/pyproject.toml" "$P/"
  for f in "${S}"/oxitest/tests/*; do [ -f "$f" ] && scrub "$f" >"$P/tests/$(basename "$f")"; done
  for f in NOTES.md CHECKERS.txt; do scrub "${S}/oxitest/$f" >"$P/$f"; done
  printf '{\n  "include": ["tests"],\n  "venvPath": "%s",\n  "venv": ".venv",\n  "pythonVersion": "3.12",\n  "extraPaths": ["../APP"]\n}\n' "$E" >"$P/pyrightconfig.json"
  printf 'Run from this folder:\n\n    PYTHONPATH=../APP %s/.venv/bin/oxitest -vv --color=never\n\nType checkers:\n\n    PYTHONPATH=../APP ty check --python %s/.venv/bin/python tests\n    nix-shell -p pyright --run pyright\n' "$E" "$E" >"$P/RUN.md"
  (cd "$P" && PYTHONPATH=../APP "$E/.venv/bin/oxitest" -vv --color=never >OUTPUT.txt 2>&1 || true)
  rm -rf "$P/.oxitest_cache"
}
build() { # $1 rater, $2 letter for the form, $3 for oxitest
  local R="$B/rater$1"
  mkdir -p "$R/APP" "$R/ORIGINAL_pytest" "$R/_review"
  (cd "${S}/flaskr-src" && tar --exclude=__pycache__ -cf - flaskr) | (cd "$R/APP" && tar -xf -)
  cp "${S}/LICENSE.flask" "$R/APP/"
  cp "${S}"/pytest/{conftest.py,data.sql,test_blog.py} "${S}/LICENSE.flask" "$R/ORIGINAL_pytest/"
  printf '{"venvPath": "%s", "venv": ".venv", "extraPaths": ["../APP"]}\n' "$E" >"$R/ORIGINAL_pytest/pyrightconfig.json"
  printf 'Run from this folder:\n\n    PYTHONPATH=../APP %s/.venv/bin/pytest -v -p no:cacheprovider\n' "$E" >"$R/ORIGINAL_pytest/RUN.md"
  (cd "$R/ORIGINAL_pytest" && PYTHONPATH=../APP "$E/.venv/bin/pytest" -v -p no:cacheprovider >OUTPUT.txt 2>&1 || true)
  form_port "${S}/form" "$R/PORT_$2"
  oxi_port "$R/PORT_$3"
}
build 1 M K
build 2 K M
build 3 M K
