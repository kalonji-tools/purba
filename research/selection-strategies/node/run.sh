#!/usr/bin/env bash
# Node's built-in runner: its default file globs mark a file by name and by
# folder, and a call to test() registers each test while the file runs.
# usage: nix-shell -p nodejs --run "bash run.sh <empty dir>"
set -u
D=$1
cd "$D" || exit 1
echo "== node --version"
node --version

mkdir -p test lib src
cat >test/helper.js <<'EOF'
// a helper in the folder test/: no call to test()
exports.one = () => 1;
EOF
cat >src/lend.test.js <<'EOF'
const test = require('node:test');
const assert = require('node:assert');
const { one } = require('../test/helper.js');

test('lends one', () => assert.strictEqual(one(), 1));

// a loop registers one test for each name, which only a run can see
for (const n of [1, 2]) {
  test(`lends ${n}`, () => assert.ok(n > 0));
}

// the same name, registered twice
test('lends one', () => assert.ok(true));
EOF
cat >lib/forgot.js <<'EOF'
// a test in a file that no default glob matches
const test = require('node:test');
test('never seen', () => { throw new Error('ran'); });
EOF
cat >src/fixtures.test.js <<'EOF'
// a helper whose name matches a glob: it registers no test
exports.shop = { name: 'shop' };
EOF

echo "== node --test, with the default globs"
node --test --test-reporter=spec 2>&1 | grep -vE '^\s*$|duration_ms' | sed 's/([0-9.]*ms)//'
