#!/usr/bin/env bash
# Run each probe under the wheel of main, then count the patch forms of the 9 pinned suites.
# usage: run.sh <venv with the wheel of main and pytest> <folder of the 9 clones>
set -euo pipefail

venv=${1}
corpus=${2}
here=$(cd "$(dirname "${0}")" && pwd)
runs=${here}/runs
mkdir -p "${runs}"

: >"${runs}/probes.txt"
for probe in "${here}"/probes/test_*.py; do
  name=${probe##*/}
  echo "=== ${name}" >>"${runs}/probes.txt"
  status=0
  (cd "${here}/probes" && PYTHONPATH="${here}/probes" "${venv}/bin/purba" "${name}") >"${runs}/out.txt" 2>&1 || status=${?}
  # The module's absolute path in an AttributeError is replaced by "probes/", and nothing else.
  sed "s|${here}/||g" "${runs}/out.txt" >>"${runs}/probes.txt"
  echo "exit ${status}" >>"${runs}/probes.txt"
done
rm "${runs}/out.txt"

"${venv}/bin/python" -I "${here}/patches.py" "${corpus}" >"${runs}/patches.txt"

cd "${corpus}"
grep -rn -E 'with (mock\.|unittest\.mock\.)?patch(\.object|\.dict|\.multiple)?\(' --include='*.py' . >"${runs}/with-patch.tmp" || true
grep -rn -E '\b(Mock|MagicMock|NonCallableMock|AsyncMock)\(' --include='*.py' . >"${runs}/mock-all.tmp" || true
grep -E '/tests?/|test_' "${runs}/mock-all.tmp" >"${runs}/mocks.tmp" || true
grep -rlE '^(import pytest|from pytest|from _pytest)' --include='*.py' . >"${runs}/pytest.tmp" || true
grep -rlE '^(from unittest import mock|from unittest.mock|import unittest.mock|import mock|from mock)' --include='*.py' . >"${runs}/mock-import.tmp" || true
built=$(wc -l <"${runs}/mocks.tmp")
specced=$(grep -c -E 'spec(_set)?=' "${runs}/mocks.tmp" || true)
pytest_files=$(wc -l <"${runs}/pytest.tmp")
mock_files=$(wc -l <"${runs}/mock-import.tmp")
{
  echo "=== grep: each 'with ...patch(' line in the 9 suites"
  cat "${runs}/with-patch.tmp"
  echo "=== grep: Mock(...) built in a test file, and how many carry spec= or spec_set= on the same line"
  echo "built: ${built}"
  echo "with a spec: ${specced}"
  echo "=== test files that import pytest, and that import unittest.mock or mock"
  echo "pytest: ${pytest_files}"
  echo "mock: ${mock_files}"
} >"${runs}/greps.txt"
rm "${runs}"/*.tmp
