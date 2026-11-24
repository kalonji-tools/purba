#!/usr/bin/env bash
# What Jest and Vitest pick as a test file, and what a helper file beside the tests does.
# usage: nix-shell -p nodejs --run "bash run.sh <dir with node_modules/jest@30.5.2 and vitest@5.1.1> <empty dir>"
set -u
DEPS=$1
D=$2
cd "$D" || exit 1
ln -s "$DEPS/node_modules" node_modules
echo '{"name":"shop","type":"module"}' >package.json
mkdir -p src/__tests__
cat >src/shop.js <<'EOF'
export const make = () => 1;
EOF
cat >src/shop.test.js <<'EOF'
import { make } from "./shop.js";
import { one } from "./__tests__/helpers.js";
const check = (got) => got === one();     // a helper beside the tests
test("lends", () => { expect(check(make())).toBe(true); });
EOF
cat >src/__tests__/helpers.js <<'EOF'
export const one = () => 1;               // a helper file in __tests__
EOF
cat >src/__tests__/lend.js <<'EOF'
import { make } from "../shop.js";
test("lends from __tests__", () => { expect(make()).toBe(1); });
EOF
cat >src/fixtures.test.js <<'EOF'
export const shop = () => ({});           // a helper file named *.test.js
EOF
echo "== jest --version"
node node_modules/jest/bin/jest.js --version
echo "== jest --listTests (default testMatch)"
node --experimental-vm-modules node_modules/jest/bin/jest.js --listTests 2>/dev/null | sed "s|$D/||" | sort
echo "== jest (run)"
node --experimental-vm-modules node_modules/jest/bin/jest.js 2>&1 | grep -E 'PASS|FAIL|must contain at least one test|^Tests:|^Test Suites:'
echo "== vitest --version"
node node_modules/vitest/vitest.mjs --version
echo "== vitest list --filesOnly (default include)"
node node_modules/vitest/vitest.mjs list --filesOnly 2>&1 | sed "s|$D/||" | sort
echo "== vitest run --globals (the test files call test() with no import)"
node node_modules/vitest/vitest.mjs run --globals 2>&1 | grep -E '✓|×|FAIL|No test suite found|Test Files|Tests ' | head -8
