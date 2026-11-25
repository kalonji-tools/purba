#!/usr/bin/env bash
# git's top level: the nearest folder that holds .git, up from the working
# folder. GIT_DIR, GIT_WORK_TREE and GIT_CEILING_DIRECTORIES change it.
# usage: bash run.sh <empty dir>
set -u
D=$1
export HOME=$D/home XDG_CONFIG_HOME=$D/home/.config GIT_CONFIG_NOSYSTEM=1
unset GIT_DIR GIT_WORK_TREE GIT_CEILING_DIRECTORIES GIT_DISCOVERY_ACROSS_FILESYSTEM
mkdir -p "$HOME" "$D/p/a/inner"
echo "== git --version"
git --version

# show <cwd> [env...]: the top level, the prefix, and the exit code
show() {
  local cwd=$1
  shift
  echo "-- cwd ${cwd#"$D"/}: $* git rev-parse --show-toplevel --show-prefix"
  (
    cd "$cwd" && env "$@" GIT_CEILING_DIRECTORIES="${CEIL:-$D}" git rev-parse --show-toplevel --show-prefix 2>&1
    echo "exit=$?"
  )
}

echo "== 1. outside git"
show "$D/p/a"
git -C "$D/p" init -q
printf 'x\n' >"$D/p/a/m.txt"
git -C "$D/p" add -A
echo "== 2. p is a repository"
show "$D/p"
show "$D/p/a"
echo "== 3. the path that ls-files prints, from p and from p/a"
(cd "$D/p" && git ls-files)
(cd "$D/p/a" && git ls-files)
(cd "$D/p/a" && git ls-files --full-name)
echo "== 4. GIT_CEILING_DIRECTORIES=p: git does not go up into p"
CEIL=$D/p show "$D/p/a"
echo "== 5. GIT_DIR=p/.git with no work tree: the working folder is the top level"
show "$D/p/a" GIT_DIR="$D/p/.git"
echo "== 6. GIT_DIR and GIT_WORK_TREE=p"
show "$D/p/a" GIT_DIR="$D/p/.git" GIT_WORK_TREE="$D/p"
echo "== 7. -C p/a from the folder above p"
(cd "$D" && GIT_CEILING_DIRECTORIES=$D git -C p/a rev-parse --show-toplevel)
echo "== 8. a nested repository p/a/inner: the nearest wins"
git -C "$D/p/a/inner" init -q
show "$D/p/a/inner"
