: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "probe: why the free-threaded Python opens the syslog socket" {
  run "${BATS_TEST_DIRNAME}/probe/who.sh"
  printf '%s\n' "${output}"
  false
}
