# .github/scripts/require-abi3.sh against the wheels it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/require-abi3.sh"
  isolate
  cd "${BATS_TEST_TMPDIR}" || return 1
  printf '[project]\nname = "purba"\nrequires-python = ">=3.12"\n' >pyproject.toml
  mkdir dist
}

# An empty file under each name, because the script reads only the name.
wheels() {
  local name
  for name in "$@"; do
    : >"dist/${name}"
  done
}

@test "abi3 wheels at the floor pass" {
  wheels purba-0.1.0-cp312-abi3-manylinux_2_34_x86_64.whl \
    purba-0.1.0-cp312-abi3-macosx_11_0_arm64.whl \
    purba-0.1.0-cp312-abi3-win_amd64.whl

  run "${script}" dist

  [[ "${status}" -eq 0 ]]
}

@test "a wheel built for one interpreter is refused, and named" {
  wheels purba-0.1.0-cp312-abi3-win_amd64.whl \
    purba-0.1.0-cp314-cp314t-manylinux_2_34_x86_64.whl

  run "${script}" dist

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"cp312-abi3"* ]]
  [[ "${output}" == *"purba-0.1.0-cp314-cp314t-manylinux_2_34_x86_64.whl"* ]]
  [[ "${output}" != *"win_amd64"* ]]
}

@test "an abi3 wheel at another floor is refused" {
  wheels purba-0.1.0-cp313-abi3-win_amd64.whl

  run "${script}" dist

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"purba-0.1.0-cp313-abi3-win_amd64.whl"* ]]
}

@test "a directory that holds no wheel exits 2" {
  run "${script}" dist

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"no wheel"* ]]
}

@test "a requires-python it cannot read exits 2" {
  printf '[project]\nrequires-python = "~=3.12"\n' >pyproject.toml
  wheels purba-0.1.0-cp312-abi3-win_amd64.whl

  run "${script}" dist

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"requires-python"* ]]
}

@test "the wrong number of arguments exits 2" {
  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"usage: require-abi3.sh <directory>"* ]]
}

# No floor is 3.0, so a floor the script can read refuses this wheel, and one it
# cannot read exits 2.
@test "the real pyproject.toml names a floor it can read" {
  cp "${BATS_TEST_DIRNAME}/../../pyproject.toml" pyproject.toml
  wheels purba-0.1.0-cp30-abi3-win_amd64.whl

  run "${script}" dist

  [[ "${status}" -eq 1 ]]
}
