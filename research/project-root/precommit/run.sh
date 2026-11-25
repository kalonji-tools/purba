#!/usr/bin/env bash
# pre-commit's root is git's top level. Outside git, pre-commit refuses.
# A hook runs from the top level, and gets paths relative to it.
# usage: PRE_COMMIT_HOME=<empty dir> bash run.sh <pre-commit 4.7.0> <empty dir>
set -u
PC=$1
D=$2
export HOME=$D/home XDG_CONFIG_HOME=$D/home/.config GIT_CONFIG_NOSYSTEM=1
export GIT_CEILING_DIRECTORIES=$D
mkdir -p "$HOME" "$D/p/a"
echo "== pre-commit --version"
"$PC" --version
cat >"$D/p/.pre-commit-config.yaml" <<'EOF'
repos:
  - repo: local
    hooks:
      - id: where
        name: where
        entry: sh -c 'echo "pwd=$(pwd)"; echo "args=$*"' --
        language: system
        verbose: true
EOF
printf 'x = 1\n' >"$D/p/a/m.py"

echo "== 1. outside git, from p/a"
(
  cd "$D/p/a" && "$PC" run --files m.py >"$D/out1" 2>&1
  echo "exit=$?"
  grep -E 'error|Error' "$D/out1"
)
echo "== 2. inside git: the hook runs from p, with a path from p"
git -C "$D/p" init -q
git -C "$D/p" add -A
(cd "$D/p/a" && "$PC" run --files m.py 2>&1 | grep -E '^(pwd|args)=')
