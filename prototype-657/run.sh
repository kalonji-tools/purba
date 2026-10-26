#!/usr/bin/env bash
# PROTOTYPE (#657), never merge. Builds the release wheel and the `fault` test
# build, then runs each defect shape and prints the line and the exit code.
set -u
out=${1:-target/proto-657}; here=$(cd "$(dirname "$0")" && pwd)
for v in off fault; do
  f=extension-module; [[ $v == fault ]] && f=extension-module,fault
  maturin build --quiet --features "$f" --out "$out/wheel-$v"
  python -m venv --clear "$out/venv-$v"
  "$out/venv-$v/bin/pip" install -q --no-index --find-links "$out/wheel-$v" purba
done
cd "$here/cases"
for f in ran-panics ran-raises ran-lies load-raises report-panics; do
  for t in test_plain.py test_catches.py; do
    echo "### PURBA_FAULT=$f $t"; PURBA_FAULT=$f "$out/venv-fault/bin/purba" $t 2>/dev/null; echo "exit=$?"
  done
done
echo "### release build, PURBA_FAULT=ran-lies"; PURBA_FAULT=ran-lies "$out/venv-off/bin/purba" test_plain.py; echo "exit=$?"
echo "### release build, a latin-1 cookie (a real defect)"; "$out/venv-off/bin/purba" test_cookie.py; echo "exit=$?"
echo "### release build, builtins.py"; (cd builtins && "$out/venv-off/bin/purba" builtins.py); echo "exit=$?"
