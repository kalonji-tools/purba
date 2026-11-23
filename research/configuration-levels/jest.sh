#!/usr/bin/env bash
# Probe: which file Jest reads, and how a timeout in the file, on the command line and in code resolve.
# Usage: jest.sh <node_modules-dir> <empty-work-dir>   (run under `mise exec node@24 --`)
# Each test sleeps 1500 ms. A timeout below that fails it.
set -u
NM=$1
W=$2
mkdir -p "$W"
cd "$W" || exit 1
export HOME="$W/home"
mkdir -p "$HOME"
unset CI CONTINUOUS_INTEGRATION BUILD_NUMBER RUN_ID GITHUB_ACTIONS
JEST="$NM/.bin/jest"
run() {
  echo "\$ (cd ${PWD#$W/}) ${JENV:-}jest $*"
  env ${JENV:-} "$JEST" --colors=false "$@" 2>&1 |
    grep -E '^(PASS|FAIL)|✓|✕|Exceeded timeout|Tests:|Multiple configurations|New snapshot was not written|written|configs' | sed "s#$W/##g" | head -8
  echo
}

mkdir -p proj/sub
cat >proj/t.test.js <<'EOF'
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
test("plain", async () => { await sleep(1500); });
test("own timeout 5000", async () => { await sleep(1500); }, 5000);
EOF
echo "## 1. the project file: testTimeout 1000 fails 'plain'; the test's own argument wins"
printf 'module.exports = { testTimeout: 1000 };\n' >proj/jest.config.js
(cd proj && run t.test.js)
echo "## 2. the command line beats the file: --testTimeout 3000"
(cd proj && run t.test.js --testTimeout 3000)
echo "## 3. code: jest.setTimeout in the test file beats the command line"
cat >proj/t2.test.js <<'EOF'
jest.setTimeout(1000);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
test("plain under setTimeout 1000", async () => { await sleep(1500); });
test("own timeout 5000", async () => { await sleep(1500); }, 5000);
EOF
(cd proj && run t2.test.js --testTimeout 3000)
rm proj/t2.test.js
echo "## 4. a package.json with no \"jest\" key stops the walk up"
printf '{"name":"sub"}\n' >proj/sub/package.json
cp proj/t.test.js proj/sub/t.test.js
(cd proj/sub && run t.test.js)
rm proj/sub/package.json
echo "   with no package.json in sub, the walk reaches proj/jest.config.js"
(cd proj/sub && run t.test.js)
rm proj/sub/t.test.js
echo "## 5. two files in one directory are refused"
printf '{"name":"p","jest":{"testTimeout":1000}}\n' >proj/package.json
(cd proj && run t.test.js)
rm proj/package.json
echo "## 6. the environment: CI decides whether a new snapshot is written"
cat >proj/s.test.js <<'EOF'
test("snap", () => { expect({ a: 1 }).toMatchSnapshot(); });
EOF
(cd proj && JENV="CI=true " run s.test.js)
(cd proj && run s.test.js)
rm -rf proj/__snapshots__
echo "   --ci=false on the command line beats CI=true"
(cd proj && JENV="CI=true " run s.test.js --ci=false)
rm -rf proj/__snapshots__
echo "## 7. no project file: Jest reads no user file, so a jest.config.js in HOME changes nothing (default 5000 ms)"
printf 'module.exports = { testTimeout: 1000 };\n' >"$HOME/jest.config.js"
rm proj/jest.config.js
echo "   with no file at all up to /, Jest stops with 'Could not find a config file'"
(cd proj && "$JEST" t.test.js 2>&1 | grep -m1 -oE 'Could not find a config file')
printf '{"name":"p"}\n' >proj/package.json
(cd proj && run t.test.js)
