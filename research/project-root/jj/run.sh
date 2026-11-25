#!/usr/bin/env bash
# Jujutsu's workspace root: the nearest folder that holds .jj, up from the
# working folder. With none, jj refuses, and hints at git only when the
# working folder itself holds .git.
# usage: bash run.sh <empty dir>
set -u
D=$1
export HOME=$D/home XDG_CONFIG_HOME=$D/home/.config GIT_CONFIG_NOSYSTEM=1
export JJ_CONFIG=$D/home/jj.toml
mkdir -p "$HOME" "$D/p/a"
printf '[user]\nname = "probe"\nemail = "probe@example.invalid"\n[ui]\ncolor = "never"\npaginate = "never"\n' >"$JJ_CONFIG"
printf 'x\n' >"$D/p/a/m.txt"
echo "== jj --version"
jj --version

# show <cwd> <args...>
show() {
  local cwd=$1
  shift
  echo "-- cwd ${cwd#"$D"/}: jj $*"
  (
    cd "$cwd" && jj "$@" 2>&1
    echo "exit=$?"
  )
}

echo "== 1. no repository"
show "$D/p/a" root
echo "== 2. p is a git repository: the hint appears from p, not from p/a"
git -C "$D/p" init -q
show "$D/p" root
show "$D/p/a" root
echo "== 3. p holds .jj, colocated with .git"
(cd "$D/p" && jj git init --colocate >/dev/null 2>&1)
show "$D/p/a" root
echo "== 4. -R p from the folder above p"
show "$D" -R p root
echo "== 5. the path that file list prints, from p and from p/a"
show "$D/p" file list
show "$D/p/a" file list
