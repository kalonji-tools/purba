: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "probe: who holds fd 1 when stdout is closed" {
  cd "${BATS_TEST_DIRNAME}/probe"
  run bash -c 'bash -c "lsof -a -p \$\$ -d 0-9 >&2" >&-
    python -c "import sys; print(sys.executable, sys._base_executable, file=sys.stderr)"
    python lsof.py >&-
    PYTHON_GIL=1 python lsof.py >&-
    python -X dev lsof.py >&-' 2>&1
  printf '%s\n' "${output}"
  false
}
