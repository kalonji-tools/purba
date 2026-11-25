#!/usr/bin/env bash
# Build three blind rater folders, laid out as the rating of selection's three
# forms was (proto/1231/ratings/build_blind.sh at 6266c8a1), with one project
# per version, and two purba ports that differ only in the build of purba.
# $1 = the eval folder. $2 and $3 = the venvs of purba's builds A and B; each
# also holds pytest, oxitest and loguru.
set -euo pipefail
E="$1"
VA="$2"
VB="$3"
B="$E/blind"
H="$(cd "$(dirname "$0")/.." && pwd)" # proto/1245/
K="$H/benchmark"
for v in "$VA" "$VB"; do
  [ -x "$v/bin/purba" ] || {
    echo "no purba in $v" >&2
    exit 1
  }
done
clean() { find "$1" \( -name __pycache__ -o -name .pytest_cache -o -name .oxitest_cache \) -prune -exec rm -rf {} +; }
copy() {
  (cd "$1" && tar --exclude=__pycache__ --exclude=.pytest_cache --exclude=.oxitest_cache --exclude=CHECKERS.txt.gz -cf - .) | (cd "$2" && tar -xf -)
  gzip -dc "$1/CHECKERS.txt.gz" >"$2/CHECKERS.txt"
}
# A venv under a neutral name, so that a path does not tell the builds apart.
link() { # $1 = the folder of the link, $2 = the venv
  mkdir -p "$(dirname "$1")"
  ln -sfn "$2" "$1"
}
pyright_config() { # $1 = the folder, $2 = the venv
  printf '{\n  "venvPath": "%s",\n  "venv": "%s",\n  "include": ["tests"]\n}\n' \
    "$(dirname "$2")" "$(basename "$2")" >"$1/pyrightconfig.json"
}

original() { # $1 = the folder, $2 = the venv
  local O="$1" V="$2"
  copy "$K/pytest" "$O"
  pyright_config "$O" "$V"
  cat >"$O/RUN.md" <<EOF
Run from this folder:

    $V/bin/pytest -v

All options: \`$V/bin/pytest --help\`.
EOF
  (cd "$O" && "$V/bin/pytest" -v -p no:cacheprovider >OUTPUT.txt 2>&1 || true)
  clean "$O"
}

oxitest_port() { # $1 = the folder, $2 = the venv
  local P="$1" V="$2"
  copy "$K/oxitest" "$P"
  pyright_config "$P" "$V"
  cat >"$P/RUN.md" <<EOF
Run from this folder (strict = "abort" comes from \`pyproject.toml\`):

    $V/bin/oxitest -vv --color=never

All options: \`$V/bin/oxitest --help\`.
EOF
  (cd "$P" && "$V/bin/oxitest" -vv --color=never >OUTPUT.txt 2>&1 || true)
  clean "$P"
}

purba_port() { # $1 = the folder, $2 = the venv, $3 = the rules (a or b)
  local P="$1" V="$2"
  copy "$K/purba" "$P"
  mkdir -p "$P/rules"
  cp "$H/rules-$3"/*.md "$P/rules/"
  pyright_config "$P" "$V"
  cat >"$P/RUN.md" <<EOF
Run from this folder. This project's \`pyproject.toml\` holds no \`[tool.purba]\`, so the run sets the variable that \`rules/ROOT.md\` names:

    PURBA_IMPLICIT_ROOT=1 $V/bin/purba

Rules of the runner: \`rules/\`, starting with \`rules/TESTS.md\`.
EOF
  (cd "$P" && PURBA_IMPLICIT_ROOT=1 "$V/bin/purba" >OUTPUT.txt 2>&1 || true)
  clean "$P"
}

build() { # $1 rater; $2 $3 $4 = the letters of oxitest, purba A and purba B
  local R="$B/rater$1" L="$E/venvs/rater$1"
  rm -rf "$R" "$L"
  mkdir -p "$R/ORIGINAL_pytest" "$R/PORT_$2" "$R/PORT_$3" "$R/PORT_$4" "$R/_review"
  link "$L/ORIGINAL" "$VA"
  link "$L/$2" "$VA"
  link "$L/$3" "$VA"
  link "$L/$4" "$VB"
  original "$R/ORIGINAL_pytest" "$L/ORIGINAL"
  oxitest_port "$R/PORT_$2" "$L/$2"
  purba_port "$R/PORT_$3" "$L/$3" a
  purba_port "$R/PORT_$4" "$L/$4" b
}
# The letters come from ratings/KEY.txt.
build 1 T R S
build 2 S T R
build 3 S R T
