# scripts/order-of-design.sh against the landscapes it refuses, and the order it writes.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../order-of-design.sh"
  make_repo
  order_refused="The order of design is refused when it differs from the order the rule calculates"
  row_refused="A risk is refused when its score or the item it lands on differs from its row"
  no_row_refused="A risk is refused when the record holds no row for it"
  unknown_refused="A name is refused when it is no part or concern of the landscape"
  circle_refused="The order of design is refused when items wait on each other in a circle"

  # Scores: beta 9, gamma 6, delta 4, the epsilon seam 2, alpha 0. Beta uses
  # gamma, and the epsilon seam crosses gamma, so neither part comes first.
  cat >LANDSCAPE.md <<'LANDSCAPE'
# purba's landscape

## The quality priorities

1. **Honest outcome.** Green means a behaviour was checked.

## The parts and their dependencies

- **alpha.** Starts the run.
- **beta.** Reads the tests.
- **gamma.** Parses the source.

```mermaid
flowchart TD
  alpha -- "a query" --> beta
  alpha -- "the source" --> gamma
  beta -- "an AST" --> gamma
```

## The invariants

- **one parser.** No part but gamma parses the source, in beta.

## The cross-cutting concerns

- **delta.** Keeps an error apart from an outcome, in alpha and beta.
- **the epsilon seam.** Carries a live object, in every part. No switch turns it off.

## The risks

- **beta misses.** A test goes unread. Lands on beta; probability high, impact high.
- **gamma refuses.** A file goes unread. Lands on gamma; probability medium, impact high.
- **delta hides.** An error looks failed. Lands on delta; probability medium, impact medium.
- **the seam leaks.** An object leaks. Lands on the epsilon seam; probability low, impact medium.

## The order of design

Each part or concern waits for the parts it uses and the concerns that cross it.

1. **delta.**
2. **the epsilon seam.**
3. **gamma.**
4. **beta.**
5. **alpha.**

[A design in depth waits for what it assumes](docs/decisions/order.md) holds why.

## What belongs in this file

Each item takes one line.
LANDSCAPE

  mkdir -p docs/decisions
  cat >docs/decisions/order.md <<'RECORD'
# A design in depth waits for what it assumes

| level | value | impact, by the quality a risk breaks |
|---|:-:|---|
| high | 3 | an honest outcome |

| risk | lands on | breaks | impact | probability | evidence |
|---|---|---|:-:|:-:|---|
| beta misses | beta | honest outcome | high | high | a count |
| gamma refuses | gamma | honest outcome | high | medium | a judgment |
| delta hides | delta | legible result | medium | medium | a count |
| the seam leaks | the epsilon seam | legible result | medium | low | a judgment |
| a slow seam | the epsilon seam | fast feedback | low | medium | stays out |
RECORD
}

# Each argument is one line of the order section, in its order.
order() {
  local name n=0 lines=""
  for name in "$@"; do
    n=$((n + 1))
    lines+="${n}. **${name}.**"$'\n'
  done
  awk -v lines="${lines}" '
    /^## / { section = ($0 == "## The order of design") }
    section && /^[0-9]+\. / { if (!written) printf "%s", lines; written = 1; next }
    { print }
  ' LANDSCAPE.md >LANDSCAPE.tmp
  mv LANDSCAPE.tmp LANDSCAPE.md
}

# Replaces the first match of a fixed string in a file, so a test names only its change.
swap() {
  local file=$1 from=$2 to=$3 text
  text=$(<"${file}")
  [[ "${text}" == *"${from}"* ]] || return 1
  printf '%s\n' "${text/"${from}"/"${to}"}" >"${file}"
}

# Removes each line of the landscape that starts with the text given.
drop() {
  local start=$1
  awk -v start="${start}" 'index($0, start) != 1' LANDSCAPE.md >LANDSCAPE.tmp
  mv LANDSCAPE.tmp LANDSCAPE.md
}

@test "a landscape whose order is the calculated order passes, and prints nothing" {
  run "${script}" --check

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a changed probability that moves an item is refused, and the calculated order is named" {
  swap LANDSCAPE.md "seam; probability low" "seam; probability high"
  swap docs/decisions/order.md "| medium | low | a judgment |" "| medium | high | a judgment |"

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${order_refused}"* ]]
  [[ "${output}" == *"calculated: the epsilon seam, gamma, delta, beta, alpha"* ]]
  [[ "${output}" != *"${row_refused}"* ]]
}

@test "a removed arrow that frees a part is refused" {
  drop '  beta -- "an AST" --> gamma'

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${order_refused}"* ]]
  [[ "${output}" == *"calculated: delta, the epsilon seam, beta, gamma, alpha"* ]]
}

@test "a concern that crosses fewer parts frees them, and the stale order is refused" {
  swap LANDSCAPE.md "object, in every part." "object, in alpha."

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${order_refused}"* ]]
  [[ "${output}" == *"calculated: gamma, delta, beta, the epsilon seam, alpha"* ]]
}

@test "a sentence of another section that ends in a list of parts is not a crossing" {
  swap LANDSCAPE.md "the source, in beta." "the source, in gamma."

  run "${script}" --check

  [[ "${status}" -eq 0 ]]
}

@test "a risk whose item differs from its row in the record is refused, and named" {
  swap LANDSCAPE.md "Lands on delta;" "Lands on beta;"

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${row_refused}"* ]]
  [[ "${output}" == *"  delta hides"* ]]
  [[ "${output}" != *"  beta misses"* ]]
}

@test "a risk whose score differs from its row in the record is refused, and named" {
  swap docs/decisions/order.md "| beta | honest outcome | high | high |" \
    "| beta | honest outcome | high | medium |"

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${row_refused}"* ]]
  [[ "${output}" == *"  beta misses"* ]]
}

@test "a risk with no row in the record is refused, and named" {
  swap LANDSCAPE.md "## The order of design" \
    $'- **alpha churns.** Lands on alpha; probability low, impact high.\n\n## The order of design'

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${no_row_refused}"* ]]
  [[ "${output}" == *"  alpha churns"* ]]
}

@test "a row of the record with no risk in the landscape passes" {
  run "${script}" --check

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"a slow seam"* ]]
}

@test "an arrow to a name that is no part is refused, and named" {
  swap LANDSCAPE.md '  beta -- "an AST" --> gamma' \
    $'  beta -- "an AST" --> gamma\n  gamma -- "a token" --> omega'

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${unknown_refused}"* ]]
  [[ "${output}" == *"  omega"* ]]
}

@test "two parts that use each other are refused" {
  swap LANDSCAPE.md '  beta -- "an AST" --> gamma' \
    $'  beta -- "an AST" --> gamma\n  gamma -- "a name" --> beta'

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${circle_refused}"* ]]
  [[ "${output}" == *"  beta"* ]]
  [[ "${output}" == *"  gamma"* ]]
}

@test "a tie in score goes to the item more waits on" {
  swap LANDSCAPE.md "seam; probability low" "seam; probability medium"
  swap docs/decisions/order.md "| medium | low | a judgment |" "| medium | medium | a judgment |"
  order "the epsilon seam" gamma delta beta alpha

  run "${script}" --check

  [[ "${status}" -eq 0 ]]
}

@test "a tie that nothing else breaks goes to the line that comes first, parts before concerns" {
  drop "- **beta misses.**"
  drop "- **gamma refuses.**"
  drop "- **delta hides.**"
  drop "- **the seam leaks.**"
  swap LANDSCAPE.md "object, in every part." "object, in alpha."
  order gamma delta beta "the epsilon seam" alpha

  run "${script}" --check

  [[ "${status}" -eq 0 ]]
}

@test "an item whose line links its design leaves the order" {
  swap LANDSCAPE.md "in alpha and beta." "in alpha and beta. [Its design](docs/decisions/delta.md)."
  drop "- **delta hides.**"
  order "the epsilon seam" gamma beta alpha

  run "${script}" --check

  [[ "${status}" -eq 0 ]]
}

@test "a designed item that keeps its order line is refused" {
  swap LANDSCAPE.md "in alpha and beta." "in alpha and beta. [Its design](docs/decisions/delta.md)."
  drop "- **delta hides.**"

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${order_refused}"* ]]
  [[ "${output}" == *"calculated: the epsilon seam, gamma, beta, alpha"* ]]
}

@test "with no option, it writes the calculated order, prints each step, and changes nothing else" {
  cp LANDSCAPE.md expected.md
  order alpha beta gamma delta "the epsilon seam"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"1. delta, by score: delta 4, the epsilon seam 2"* ]]
  [[ "${output}" == *"5. alpha"* ]]
  diff expected.md LANDSCAPE.md

  run "${script}" --check

  [[ "${status}" -eq 0 ]]
}

@test "an order list whose numbers do not count from one is refused, and written with no option" {
  swap LANDSCAPE.md "1. **delta.**" "0. **delta.**"

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${order_refused}"* ]]

  run "${script}"

  [[ "${status}" -eq 0 ]]
  grep -qxF "1. **delta.**" LANDSCAPE.md

  run "${script}" --check

  [[ "${status}" -eq 0 ]]
}

@test "with no option, an order section with no numbered list cannot be written" {
  drop "1. **delta.**"
  drop "2. **the epsilon seam.**"
  drop "3. **gamma.**"
  drop "4. **beta.**"
  drop "5. **alpha.**"

  run "${script}"

  [[ "${status}" -eq 2 ]]
}

@test "a link to a section of the record still finds the record" {
  swap LANDSCAPE.md "(docs/decisions/order.md)" "(docs/decisions/order.md#decision-outcome)"

  run "${script}" --check

  [[ "${status}" -eq 0 ]]
}

@test "a landscape with no order section cannot be checked" {
  swap LANDSCAPE.md "## The order of design" "## The sequence"

  run "${script}" --check

  [[ "${status}" -eq 2 ]]
}

@test "an order section that links no record cannot be checked" {
  swap LANDSCAPE.md "(docs/decisions/order.md)" "(https://example.invalid/order)"

  run "${script}" --check

  [[ "${status}" -eq 2 ]]
}
