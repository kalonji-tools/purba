#!/usr/bin/env bash
# The scenarios of #1230, in purba's grammar and in oxitest's -E. Prototype; this
# branch is never merged. Each script that runs them sources this file.
#
# After it is sourced, KEYS[i], PURBA[i] and OXITEST[i] hold one scenario. An
# empty OXITEST[i] means oxitest has no form for it, and the control skips it.

KEYS=()
PURBA=()
OXITEST=()

scenario() {
  KEYS+=("$1")
  PURBA+=("$2")
  OXITEST+=("$3")
}

# repeat <n> <text>: <text> written <n> times.
repeat() {
  local out='' i
  for ((i = 0; i < $1; i++)); do
    out+=$2
  done
  printf '%s' "$out"
}

# joined <n> <term> <operator> <last>: <n> - 1 copies of <term>, then <last>,
# joined by <operator>.
joined() {
  local out='' i
  for ((i = 1; i < $1; i++)); do
    out+="$2 $3 "
  done
  printf '%s' "$out$4"
}

# The questions of section 4 of #1226.
scenario q1-one-pick 'tag(slow)' 'mark(slow)'
scenario q2-union 'tag(slow) | tag(db)' 'mark(slow) | mark(db)'
scenario q3-exclusion 'tag(slow) & !tag(db)' 'mark(slow) & !mark(db)'
scenario q4-id-and-tag 'tests/store & tag(slow)' 'source("tests/store/") & mark(slow)'
scenario q5-intersection 'tag(slow) & tag(db)' 'mark(slow) & mark(db)'
scenario q6-tag-or-id 'tag(slow) | tests/api/test_admin.py' 'mark(slow) | source("tests/api/test_admin.py")'
scenario q7-not-both '!(tag(slow) & tag(db))' '!(mark(slow) & mark(db))'
scenario q8-either-not-both '(tag(slow) | tag(db)) & !(tag(slow) & tag(db))' \
  '(mark(slow) | mark(db)) & !(mark(slow) & mark(db))'
scenario q9-no-tag '!tag(slow) & !tag(db) & !tag(flaky)' '!mark()'

# Precedence.
scenario precedence 'tag(db) | tag(slow) & !tag(db)' 'mark(db) | mark(slow) & !mark(db)'
scenario precedence-grouped 'tag(db) | (tag(slow) & !tag(db))' 'mark(db) | (mark(slow) & !mark(db))'

# Ids and prefixes of ids.
scenario id-dir 'tests/api' 'source("tests/api/")'
scenario id-file 'tests/api/test_user.py' 'source("tests/api/test_user.py")'
scenario id-test 'tests/api/test_user.py::test_add' 'name(="tests/api/test_user.py::test_add")'
scenario id-case 'tests/api/test_user.py::test_add::neg' 'name("tests/api/test_user.py::test_add::neg")'
scenario id-typo-issue 'store::migrat' 'name(store::migrat)'
scenario id-typo-test 'tests/store/test_store.py::test_migrat' 'name("tests/store/test_store.py::test_migrat")'
scenario id-typo-dir 'tests/stor' 'source("tests/stor")'

# Names, and a tag that no test carries.
scenario name-add 'name(add)' 'name(add)'
scenario name-math 'name(math)' 'name(math)'
scenario tag-typo 'tag(slwo)' 'mark(slwo)'
scenario empty-combination 'tag(flaky) & tests/api' 'mark(flaky) & source("tests/api/")'

# Errors of the parse.
scenario unclosed 'tag(slow) & (tag(db)' 'mark(slow) & (mark(db)'
scenario missing-operand 'tag(slow) &' 'mark(slow) &'
scenario empty-query '' ''

# Depth: <n> pairs of parentheses, and <n> negations.
for n in 255 256 257 10000; do
  scenario "parens-$n" "$(repeat "$n" '(')tag(slow)$(repeat "$n" ')')" \
    "$(repeat "$n" '(')mark(slow)$(repeat "$n" ')')"
done
for n in 255 256; do
  scenario "bangs-$n" "$(repeat "$n" '!')tag(slow)" "$(repeat "$n" '!')mark(slow)"
done

# Width: 256 terms in one & or |. The right answer of the & row is slow and not
# flaky; of the | row, every test that is not flaky, or is slow.
scenario width-256-and "$(joined 256 '!tag(flaky)' '&' 'tag(slow)')" \
  "$(joined 256 '!mark(flaky)' '&' 'mark(slow)')"
scenario width-256-or "$(joined 256 '!tag(flaky)' '|' 'tag(slow)')" \
  "$(joined 256 '!mark(flaky)' '|' 'mark(slow)')"

# Aliases. oxitest has none, so the control skips these rows.
scenario alias 'slow_store' ''
scenario alias-with-argument 'stable(slow_store)' ''
scenario alias-with-query 'stable(tag(slow))' ''
scenario alias-unknown-word 'nosuch' ''
scenario alias-unknown-call 'unstable(tag(slow))' ''
scenario alias-names-itself 'loop' ''
scenario alias-cycle 'ping' ''

# Stage one: how many files a query parses.
scenario path-and-tag 'path(tests/api/**) & tag(slow)' 'source(/^tests.api./) & mark(slow)'
scenario path-only 'path(tests/api/**)' 'source(/^tests.api./)'
scenario not-path-and-tag '!path(tests/api/**) & tag(slow)' '!source(/^tests.api./) & mark(slow)'
scenario tag-or-path 'tag(slow) | path(tests/api/**)' 'mark(slow) | source(/^tests.api./)'

# The failures of a run, as one query, run again. The failing tests of the suite
# are these two.
scenario rerun-failures 'tests/api/test_user.py::test_remove | tests/store/test_store.py::test_load' \
  'name(="tests/api/test_user.py::test_remove") | name(="tests/store/test_store.py::test_load")'
