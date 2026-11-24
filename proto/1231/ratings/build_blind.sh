#!/usr/bin/env bash
# Build three blind rater folders, laid out as the loguru round of the 1015
# rating was (prototypes/1015-form/benchmark/ratings/build_blind_loguru.sh at
# 107b09a6), with one project per version and purba's rules sheets.
# $1 = the eval folder; it holds the venv at $1/.venv.
set -euo pipefail
E="$1"
B="$E/blind"
H="$(cd "$(dirname "$0")/.." && pwd)" # proto/1231/
K="$H/benchmark"
[ -x "$E/.venv/bin/python" ] || {
  echo "make $E/.venv first" >&2
  exit 1
}
clean() { find "$1" \( -name __pycache__ -o -name .pytest_cache -o -name .oxitest_cache \) -prune -exec rm -rf {} +; }
copy() {
  (cd "$1" && tar --exclude=__pycache__ --exclude=.pytest_cache --exclude=.oxitest_cache --exclude=CHECKERS.txt.gz -cf - .) | (cd "$2" && tar -xf -)
  gzip -dc "$1/CHECKERS.txt.gz" >"$2/CHECKERS.txt"
}
pyright_config() { printf '{\n  "venvPath": "%s",\n  "venv": ".venv",\n  "include": ["tests"]\n}\n' "$E" >"$1/pyrightconfig.json"; }

original() { # $1 = the folder
  local O="$1"
  copy "$K/pytest" "$O"
  pyright_config "$O"
  cat >"$O/RUN.md" <<EOF
Run from this folder:

    $E/.venv/bin/pytest -v

Only the tests marked smoke:

    $E/.venv/bin/pytest -v -m smoke

Only the tests that failed in the last run:

    $E/.venv/bin/pytest -v --lf
EOF
  (cd "$O" && "$E/.venv/bin/pytest" -v -p no:cacheprovider >OUTPUT.txt 2>&1 || true)
  clean "$O"
}

oxitest_port() { # $1 = the folder
  local P="$1"
  copy "$K/oxitest" "$P"
  pyright_config "$P"
  cat >"$P/RUN.md" <<EOF
Run from this folder (strict = "abort" comes from \`pyproject.toml\`):

    $E/.venv/bin/oxitest -vv --color=never

Only the tests marked smoke:

    $E/.venv/bin/oxitest -vv --color=never -E 'mark(smoke)'

Only the tests that failed in the last run:

    $E/.venv/bin/oxitest -vv --color=never --lf

All options: \`$E/.venv/bin/oxitest --help\`.
EOF
  (cd "$P" && "$E/.venv/bin/oxitest" -vv --color=never >OUTPUT.txt 2>&1 || true)
  clean "$P"
}

purba_port() { # $1 = the folder
  local P="$1"
  copy "$K/purba" "$P"
  mkdir -p "$P/rules"
  cp "$H"/rules/*.md "$P/rules/"
  pyright_config "$P"
  cat >"$P/RUN.md" <<EOF
Run from this folder:

    $E/.venv/bin/purba tests

Only the tests tagged smoke:

    $E/.venv/bin/purba 'tag(smoke)'

Only the tests that failed: copy the query that the last line of a run prints, as \`rules/RERUN.md\` shows.

Rules of the runner: \`rules/\`, starting with \`rules/TESTS.md\`.
EOF
  (cd "$P" && "$E/.venv/bin/purba" tests >OUTPUT.txt 2>&1 || true)
  clean "$P"
}

build() { # $1 rater, $2 letter for oxitest, $3 letter for purba
  local R="$B/rater$1"
  rm -rf "$R"
  mkdir -p "$R/ORIGINAL_pytest" "$R/PORT_$2" "$R/PORT_$3" "$R/_review"
  original "$R/ORIGINAL_pytest"
  oxitest_port "$R/PORT_$2"
  purba_port "$R/PORT_$3"
}
build 1 R T
build 2 T R
build 3 R T
