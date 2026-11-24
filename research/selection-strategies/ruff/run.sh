#!/usr/bin/env bash
# Ruff's file set: a default list of globs, which `include` replaces, and
# `extend-exclude` narrows. A path given on the command line is checked anyway.
# usage: bash run.sh <ruff 0.17.6> <empty dir>
set -u
RUFF=$1
D=$2
cd "$D" || exit 1
echo "== ruff --version"
"$RUFF" --version

mkdir -p proj/tests proj/src proj/scripts && cd proj || exit 1
printf 'def test_a(): pass\n' >tests/test_a.py
printf 'def make(): pass\n' >tests/helper.py
printf 'x = 1\n' >src/a.py
printf 'x: int\n' >src/a.pyi
printf '#!/usr/bin/env python3\nprint(1)\n' >scripts/run
chmod +x scripts/run
show() { "$RUFF" check --show-files "$@" 2>&1 | sed "s#$D/proj/##" | sort; }

echo "== 1. the default file set"
printf '[tool.ruff]\n' >pyproject.toml
show .
echo "== 2. include = [\"tests/test_*.py\"] replaces the default"
printf '[tool.ruff]\ninclude = ["tests/test_*.py"]\n' >pyproject.toml
show .
echo "== 3. extend-exclude = [\"tests\"] narrows the default"
printf '[tool.ruff]\nextend-exclude = ["tests"]\n' >pyproject.toml
show .
echo "== 4. the same setting, with tests/helper.py and scripts/run named on the command line"
show tests/helper.py scripts/run
echo "== 5. the same, with force-exclude = true"
printf '[tool.ruff]\nextend-exclude = ["tests"]\nforce-exclude = true\n' >pyproject.toml
show tests/helper.py scripts/run

echo "== 6. D103 (a public function with no docstring) on a test named test_… and on one named lend"
mkdir -p ../d103 && cd ../d103 || exit 1
printf '"""Tests."""\n\n\ndef test_lend():\n    assert 1\n\n\ndef lend():\n    assert 1\n\n\ndef _helper():\n    pass\n' >test_shop.py
"$RUFF" check --isolated --select D103 --output-format concise test_shop.py 2>&1 | sed "s#$D/d103/##"
