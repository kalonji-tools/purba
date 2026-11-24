#!/usr/bin/env bash
# pre-commit picks each hook's files by a regex of paths (files, exclude) and by a
# file type that identify reads from the name, the mode and the shebang (types).
# usage: bash run.sh <pre-commit 4.7.0> <empty dir>
set -u
PC=$1
D=$2
cd "$D" || exit 1
echo "== pre-commit --version"
"$PC" --version

git init -q repo && cd repo || exit 1
git config user.name probe
git config user.email probe@invalid
mkdir -p tests src scripts
printf 'def test_a(): pass\n' >tests/test_a.py
printf 'def make(): pass\n' >tests/helper.py
printf 'x = 1\n' >src/a.py
printf '#!/usr/bin/env python3\nprint(1)\n' >scripts/run
chmod +x scripts/run
printf 'text\n' >notes.txt
cat >.pre-commit-config.yaml <<'EOF'
repos:
  - repo: local
    hooks:
      - {id: by-type, name: by-type, entry: echo, language: system, types: [python], verbose: true}
      - {id: by-folder, name: by-folder, entry: echo, language: system, files: ^tests/, verbose: true}
      - {id: by-name, name: by-name, entry: echo, language: system, files: '(^|/)test_[^/]*\.py$', verbose: true}
      - {id: type-minus-folder, name: type-minus-folder, entry: echo, language: system, types: [python], exclude: ^tests/, verbose: true}
      - {id: matches-none, name: matches-none, entry: echo, language: system, files: ^nothing/, verbose: true}
EOF
git add -A
echo "== identify's tags for scripts/run"
"$(dirname "$PC")/identify-cli" scripts/run
echo "== pre-commit run --all-files: the files each hook receives"
"$PC" run --all-files 2>&1 | grep -vE '^- (hook id|duration)|^$' | sed 's/\.\.\.*/ /'
