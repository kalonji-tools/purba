# scripts/crate-pain.jq, the formula that crate-verdict.sh includes.
: "${BATS_TEST_DIRNAME:?set by bats}"

# The pain and the needs that the formula gives for each entry on standard input.
formula() {
  jq -r -L "${BATS_TEST_DIRNAME}/.." 'include "crate-pain";
    fourteenths(.) as $sum | "\(hundredths($sum)) \(needs($sum))"'
}

@test "pain is half the sum of blast area and the mean of the other seven levels" {
  run formula <<<'{"blast-area": 1, "upkeep": 1, "maintainers": 1, "upstream-review": 3,
    "tests": 1, "unsafe": 1, "api-churn": 1, "owner-in-purba": 3}'

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "1.29 2" ]]
}

@test "pain 2.00 needs relief 3, because relief must exceed pain" {
  run formula <<<'{"blast-area": 2, "upkeep": 2, "maintainers": 2, "upstream-review": 2,
    "tests": 2, "unsafe": 2, "api-churn": 2, "owner-in-purba": 2}'

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "2.00 3" ]]
}

@test "a crate at pain 3.00 never joins" {
  run formula <<<'{"blast-area": 3, "upkeep": 3, "maintainers": 3, "upstream-review": 3,
    "tests": 3, "unsafe": 3, "api-churn": 3, "owner-in-purba": 3}'

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "3.00 never" ]]
}
