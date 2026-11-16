#!/usr/bin/env bash
# Count, in the test files of the pinned suites under $1, the files that import each
# module, and each pytest name that the files read.
# A test file is test_*.py or *_test.py. An import is a line that matches
# ^\s*(import|from) <module>\b, so `import pytest as pt` counts and importlib does not.
set -u
cd "$1" || exit 1
list=$(mktemp)
find . \( -name 'test_*.py' -o -name '*_test.py' \) | sort >"${list}"
echo "test files: $(wc -l <"${list}")"
for m in pytest unittest hypothesis freezegun nose nose2 testtools ward; do
  n=$(xargs grep -lE "^\s*(import ${m}\b|from ${m}\b)" <"${list}" | wc -l)
  echo "${m} ${n}"
done
echo "--- pytest names read"
xargs grep -ohE '\bpytest\.[A-Za-z_]+(\.[A-Za-z_]+)?' <"${list}" | sort | uniq -c | sort -rn
rm -f "${list}"
