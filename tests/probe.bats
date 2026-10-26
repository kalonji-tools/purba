: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "probe: who opens fd 1 when the shell closed it" {
  run "${BATS_TEST_DIRNAME}/probe/who.sh"
  printf '%s\n' "${output}"
  false
}
