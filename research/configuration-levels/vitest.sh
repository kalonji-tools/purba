#!/usr/bin/env bash
# Probe: which file Vitest reads, and how a timeout in the file, on the command line and in code resolve.
# Usage: vitest.sh <node_modules-dir> <empty-work-dir-below-that-node_modules-parent>
#        (run under `mise exec node@24 --`; the work dir sits below the node_modules so that
#        `import ... from "vitest"` resolves)
# Each test sleeps 1500 ms. A timeout below that fails it.
set -u
NM=$1
W=$2
mkdir -p "$W"
cd "$W" || exit 1
unset CI CONTINUOUS_INTEGRATION BUILD_NUMBER RUN_ID GITHUB_ACTIONS
VT="$NM/.bin/vitest"
run() {
  echo "\$ (cd ${PWD#$W/}) ${VENV:-}vitest run $*"
  env ${VENV:-} "$VT" run --no-color "$@" 2>&1 |
    grep -E '✓|×|Test timed out|Tests +[0-9]|only|Snapshots|not written|Error:' | sed "s#$W/##g" | head -8
  echo
}

mkdir -p proj/sub
cat >proj/t.test.mjs <<'EOF'
import { test } from "vitest";
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
test("plain", async () => { await sleep(1500); });
test("own timeout 5000", async () => { await sleep(1500); }, 5000);
EOF
echo "## 1. the project file: testTimeout 1000 fails 'plain'; the test's own argument wins"
printf 'export default { test: { testTimeout: 1000 } };\n' >proj/vitest.config.mjs
(cd proj && run t.test.mjs)
echo "## 2. the command line beats the file: --testTimeout 3000"
(cd proj && run t.test.mjs --testTimeout 3000)
echo "## 3. code: vi.setConfig in the test file beats the command line"
cat >proj/t2.test.mjs <<'EOF'
import { test, vi } from "vitest";
vi.setConfig({ testTimeout: 1000 });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
test("plain under setConfig 1000", async () => { await sleep(1500); });
test("own timeout 5000", async () => { await sleep(1500); }, 5000);
EOF
(cd proj && run t2.test.mjs --testTimeout 3000)
rm proj/t2.test.mjs
echo "## 4. no walk up: from proj/sub, proj/vitest.config.mjs is not read"
cp proj/t.test.mjs proj/sub/t.test.mjs
(cd proj/sub && run t.test.mjs)
echo "   --root .. names the directory"
(cd proj/sub && run --root .. sub/t.test.mjs)
rm proj/sub/t.test.mjs
echo "## 5. vitest.config beats vite.config in one directory"
printf 'export default { test: { testTimeout: 3000 } };\n' >proj/vite.config.mjs
(cd proj && run t.test.mjs)
rm proj/vite.config.mjs
echo "## 6. the environment: CI turns test.only into a failure"
cat >proj/o.test.mjs <<'EOF'
import { test } from "vitest";
test.only("only", () => {});
test("other", () => {});
EOF
(cd proj && run o.test.mjs)
(cd proj && VENV="CI=true " run o.test.mjs)
(cd proj && VENV="CI=true " run o.test.mjs --allowOnly)
rm proj/o.test.mjs
echo "## 7. the environment: CI and a new snapshot"
cat >proj/s.test.mjs <<'EOF'
import { test, expect } from "vitest";
test("snap", () => { expect({ a: 1 }).toMatchSnapshot(); });
EOF
(cd proj && VENV="CI=true " run s.test.mjs)
rm -rf proj/__snapshots__
(cd proj && run s.test.mjs)
rm -rf proj/__snapshots__
