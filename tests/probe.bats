: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "probe: what fd 1 is when stdout is closed" {
  cd "${BATS_TEST_DIRNAME}/probe"
  run bash -c 'python -VV >&2
    python fd1.py >&-
    python fd1.py import >&-
    purba test_fd1.py >&-
    echo "purba exit $?" >&2
    head -1 "$(command -v purba)" >&2' 2>&1
  printf '%s\n' "${output}"
  false
}
