#!/usr/bin/env bash
# Probe: uv's system, user and project files, its environment variables and its flags.
# Usage: uv.sh <empty-work-dir>
# It reads the resolved settings with the hidden flag --show-settings.
set -u
W=$1
mkdir -p "$W"
cd "$W" || exit 1
export HOME="$W/home" XDG_CONFIG_HOME="$W/home/.config" XDG_CONFIG_DIRS="$W/etc-xdg"
mkdir -p "$XDG_CONFIG_HOME/uv" "$XDG_CONFIG_DIRS/uv" proj/sub
for v in $(env | grep -o '^UV_[A-Z_]*'); do unset "$v"; done
printf 'anyio\n' >r.in
cp r.in proj/r.in
cp r.in proj/sub/r.in
show() {
  echo "\$ (cd ${PWD#$W/}) ${UVENV:-}uv pip compile $* --show-settings"
  env ${UVENV:-} uv pip compile --offline --show-settings "$@" r.in 2>&1 |
    grep -E '^\s+resolution:|^warning|^error|not allowed|Packages\(|^\s+"[a-z]+",$' | tr -s ' ' | tr '\n' ' '
  echo
  echo
}

echo "## 1. system file only (\$XDG_CONFIG_DIRS/uv/uv.toml)"
printf 'resolution = "lowest"\nno-build-package = ["fromsystem"]\n' >"$XDG_CONFIG_DIRS/uv/uv.toml"
(cd proj && show)
echo "## 2. plus a user file (\$XDG_CONFIG_HOME/uv/uv.toml): scalar from user, array concatenated"
printf 'resolution = "lowest-direct"\nno-build-package = ["fromuser"]\n' >"$XDG_CONFIG_HOME/uv/uv.toml"
(cd proj && show)
echo "## 3. plus a pyproject.toml with no [tool.uv] in proj: skipped, the walk goes on"
printf '[project]\nname = "p"\nversion = "0"\n' >proj/pyproject.toml
(cd proj && show)
echo "## 4. plus a project uv.toml in proj: scalar from project, array project+user+system"
printf 'resolution = "highest"\nno-build-package = ["fromproject"]\n' >proj/uv.toml
(cd proj && show)
(cd proj/sub && show)
echo "## 5. a [tool.uv] beside the uv.toml in the same directory: uv.toml wins, with a warning"
printf '[project]\nname = "p"\nversion = "0"\n[tool.uv]\nresolution = "lowest"\n' >proj/pyproject.toml
(cd proj && show)
printf '[project]\nname = "p"\nversion = "0"\n' >proj/pyproject.toml
echo "## 6. a nested sub/uv.toml: with proj/pyproject.toml a project, uv starts at the project, not at the cwd"
printf 'no-build-package = ["fromsub"]\n' >proj/sub/uv.toml
(cd proj/sub && show)
echo "   with no project, uv starts at the cwd: first match wins, proj/uv.toml is not merged"
mv proj/pyproject.toml proj/pyproject.toml.off
(cd proj/sub && show)
mv proj/pyproject.toml.off proj/pyproject.toml
rm proj/sub/uv.toml
echo "## 7. environment beats every file; the command line beats the environment"
(cd proj && UVENV="UV_RESOLUTION=lowest " show)
(cd proj && UVENV="UV_RESOLUTION=lowest " show --resolution lowest-direct)
echo "   an array from the environment and from the command line: added to the files, or replacing them?"
lock() {
  echo "\$ (cd ${PWD#$W/}) ${UVENV:-}uv lock $* --show-settings"
  env ${UVENV:-} uv lock --offline --show-settings "$@" 2>&1 |
    grep -E '^\s+no_build_package:|^error|^\s+"?[a-z]+"?,$' | tr -s ' ' | tr '\n' ' '
  echo
  echo
}
echo "   (uv pip compile takes no --no-build-package, so this uses uv lock)"
(cd proj && lock)
(cd proj && UVENV="UV_NO_BUILD_PACKAGE=fromenv " lock)
(cd proj && UVENV="UV_NO_BUILD_PACKAGE=fromenv " lock --no-build-package fromcli)
echo "## 8. --no-config, UV_NO_CONFIG, --config-file, UV_NO_SYSTEM_CONFIG"
(cd proj && show --no-config)
(cd proj && UVENV="UV_NO_CONFIG=1 " show)
printf 'resolution = "lowest"\n' >named.toml
(cd proj && show --config-file ../named.toml)
(cd proj && UVENV="UV_NO_SYSTEM_CONFIG=1 " show)
echo "## 9. a field only a project may hold, in the user file"
printf 'managed = true\n' >>"$XDG_CONFIG_HOME/uv/uv.toml"
(cd proj && show)
echo "   the same field in a project uv.toml"
sed -i '/managed/d' "$XDG_CONFIG_HOME/uv/uv.toml"
printf 'managed = true\n' >>proj/uv.toml
(cd proj && show)
sed -i '/managed/d' proj/uv.toml
echo "## 10. uv tool reads no project file: compare the resolution uv tool sees"
(cd proj && uv tool list --show-settings 2>&1 | grep -E '^\s+resolution:' | head -1)
