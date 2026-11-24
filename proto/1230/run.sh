#!/usr/bin/env bash
# Prototype harness for #1230. This branch is never merged.
#
#   proto/1230/run.sh <purba> <out-dir> <work-dir> <suites-dir>
#
# <purba> is the console script of an installed wheel. For each parser, it runs
# each scenario of scenarios.sh on a copy of suite/ with PURBA_PROTO_LIST=1,
# which prints the picks and imports nothing; then the depth rows with the limit
# off; then two queries on each suite under <suites-dir>, which purba only
# parses; then real runs of the wheel on the copy of suite/.
set -uo pipefail

purba=$1
out=$2
work=$3
suites=$4
here=$(cd "$(dirname "$0")" && pwd)
# shellcheck source=scenarios.sh
source "$here/scenarios.sh"
mkdir -p "$out/suites" "$out/wheel" "$work"
rm -rf "$work/suite"
cp -r "$here/suite" "$work/suite"

# shown <query>: the query, or its ends when it is long.
shown() {
  if ((${#1} > 160)); then
    printf '%s … %s (%s characters)' "${1:0:80}" "${1: -60}" "${#1}"
  else
    printf '%s' "$1"
  fi
}

# go <file> <dir> <query> <env...>: runs purba in <dir> on <query>.
go() {
  local file=$1 dir=$2 query=$3
  shift 3
  local status
  (cd "$dir" && env -i PATH=/run/current-system/sw/bin:/usr/bin:/bin HOME="$work" "$@" \
    timeout 120 "$purba" "$query" >"$work/stdout" 2>"$work/stderr")
  status=$?
  {
    echo "purba: ${LABEL:-$purba}"
    echo "env: $*"
    local place=${dir/#$work/<tmp>}
    echo "command: cd ${place/#$suites/<suites>} && purba '$(shown "$query")'"
    echo "--- stdout"
    if [[ -n ${COUNT_ONLY:-} ]]; then
      # A suite's test names are its own, typos included, so only the counts.
      echo "$(grep -c '' "$work/stdout") lines, $(grep -c ' (refused)$' "$work/stdout") of them refused"
    else
      cut -c1-300 "$work/stdout"
    fi
    echo "--- stderr"
    cut -c1-300 "$work/stderr" | head -40
    echo "--- exit $status"
  } >"$file"
}

for parser in pest chumsky; do
  for i in "${!KEYS[@]}"; do
    go "$out/${KEYS[$i]}.$parser.txt" "$work/suite" "${PURBA[$i]}" \
      PURBA_PROTO_LIST=1 "PURBA_PROTO_PARSER=$parser"
  done
  # Linux refuses one argument longer than 131072 bytes, so 60000 is the
  # deepest pair of parentheses one argument holds.
  for n in 3000 10000 60000; do
    go "$out/parens-$n-off.$parser.txt" "$work/suite" "$(repeat "$n" '(')tag(slow)$(repeat "$n" ')')" \
      PURBA_PROTO_LIST=1 "PURBA_PROTO_PARSER=$parser" PURBA_PROTO_DEPTH=off
    go "$out/bangs-$n-off.$parser.txt" "$work/suite" "$(repeat "$n" '!')tag(slow)" \
      PURBA_PROTO_LIST=1 "PURBA_PROTO_PARSER=$parser" PURBA_PROTO_DEPTH=off
  done
  # A wide | of ids, the shape of a long list of failures. Each term is about
  # 40 bytes, so 10000 of them pass the limit of one argument.
  for n in 2000 10000; do
    wide=$(joined "$n" 'tests/api/test_user.py::test_remove' '|' 'tests/store/test_store.py::test_load')
    go "$out/width-$n-ids.$parser.txt" "$work/suite" "$wide" \
      PURBA_PROTO_LIST=1 "PURBA_PROTO_PARSER=$parser"
  done
done

# The 9 suites: files found and parsed. purba only parses them.
for dir in "$suites"/*/; do
  name=$(basename "$dir")
  first=$(cd "$dir" && find . \( -name '.*' -a ! -name . -o -name __pycache__ \) -prune -o -type f \
    \( -name 'test_*.py' -o -name '*_test.py' \) -print | sed 's#^\./##' | LC_ALL=C sort | head -1)
  for parser in pest chumsky; do
    COUNT_ONLY=1 go "$out/suites/$name.all.$parser.txt" "$dir" 'name(test)' \
      PURBA_PROTO_LIST=1 "PURBA_PROTO_PARSER=$parser"
    COUNT_ONLY=1 go "$out/suites/$name.one.$parser.txt" "$dir" "path($first) & name(test)" \
      PURBA_PROTO_LIST=1 "PURBA_PROTO_PARSER=$parser"
  done
  # The suite's commit, for the record.
  echo "$name $(git -C "$dir" rev-parse HEAD)" >>"$out/suites/commits.txt.new"
done
LC_ALL=C sort -u "$out/suites/commits.txt.new" >"$out/suites/commits.txt"
rm -f "$out/suites/commits.txt.new"

# Real runs on the wheel: the tests run.
for parser in pest chumsky; do
  rm -rf "$work/suite"
  cp -r "$here/suite" "$work/suite"
  go "$out/wheel/all.$parser.txt" "$work/suite" 'tests' "PURBA_PROTO_PARSER=$parser"
  rerun=$(sed -n 's/^purba: proto 1230: the failures as one query: //p' "$out/wheel/all.$parser.txt")
  go "$out/wheel/rerun.$parser.txt" "$work/suite" "$rerun" "PURBA_PROTO_PARSER=$parser"
  go "$out/wheel/file.$parser.txt" "$work/suite" 'tests/math/test_math.py' "PURBA_PROTO_PARSER=$parser"
  go "$out/wheel/slow-not-flaky.$parser.txt" "$work/suite" 'tag(slow) & !tag(flaky)' \
    "PURBA_PROTO_PARSER=$parser"
  go "$out/wheel/stable-alias.$parser.txt" "$work/suite" 'stable(slow_store)' \
    "PURBA_PROTO_PARSER=$parser"
  go "$out/wheel/typo.$parser.txt" "$work/suite" 'store::migrat' "PURBA_PROTO_PARSER=$parser"
done
echo "runs in $out"
