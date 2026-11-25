#!/usr/bin/env bash
# npm's local prefix: the nearest folder that holds package.json or
# node_modules, up from the working folder; else the working folder itself.
# usage: bash run.sh <empty dir>
set -u
D=$1
export HOME=$D/home npm_config_cache=$D/home/.npm npm_config_update_notifier=false
unset npm_config_prefix
mkdir -p "$HOME" "$D/p/a/b"
echo "== npm --version"
npm --version

# npm masks a part of a path that looks like a token, so the printed path
# cannot match "$D". Each folder of the probe starts at <dir>/p.
mask() { sed -E 's#^/.*(/p(/|$))#<dir>\1#'; }

show() {
  echo "-- cwd ${1#"$D"/}: npm prefix"
  (
    cd "$1" && npm prefix >"$D/out" 2>&1
    echo "exit=$?"
    mask <"$D/out"
  )
}

echo "== 1. no marker anywhere: the working folder"
show "$D/p/a/b"
echo "== 2. p/a/node_modules"
mkdir -p "$D/p/a/node_modules"
show "$D/p/a/b"
echo "== 3. p/package.json is farther than p/a/node_modules"
printf '{"name": "p"}\n' >"$D/p/package.json"
show "$D/p/a/b"
echo "== 4. p/package.json lists p/a as a workspace, and p/a/package.json exists"
printf '{"name": "p", "workspaces": ["a"]}\n' >"$D/p/package.json"
printf '{"name": "a"}\n' >"$D/p/a/package.json"
show "$D/p/a/b"
echo "== 5. --prefix sets it"
(cd "$D/p/a/b" && npm prefix --prefix "$D/p/a/b" | mask)
