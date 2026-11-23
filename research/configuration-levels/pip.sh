#!/usr/bin/env bash
# Probe: the order of pip's files, its environment variables and its flags.
# Usage: pip.sh <venv-dir> <empty-work-dir>
# pip prints the index it uses in "Looking in indexes: ...". Each index is a path that
# does not exist, so pip fails at once and never reaches the network.
set -u
V=$1
W=$2
mkdir -p "$W"
cd "$W" || exit 1
export HOME="$W/home" XDG_CONFIG_HOME="$W/home/.config" XDG_CONFIG_DIRS="$W/etc-xdg"
mkdir -p "$XDG_CONFIG_HOME/pip" "$HOME/.pip" "$XDG_CONFIG_DIRS/pip"
for v in $(env | grep -o '^PIP_[A-Z_]*'); do unset "$v"; done
export PIP_DISABLE_PIP_VERSION_CHECK=1
SITE="$V/pip.conf"
cleanup_site() { rm -f "$SITE"; }
trap cleanup_site EXIT
idx() {
  echo "\$ ${PENV:-}pip install $*"
  env ${PENV:-} "$V/bin/python" -I -m pip install --dry-run --no-deps "$@" nothing-here 2>&1 |
    grep -E 'Looking in|^ERROR: (Invalid|Could not)' | head -1 | sed "s#file://$W/##g"
  echo
}
conf() { printf '[global]\nindex-url = file://%s/%s\n' "$W" "$1"; }

echo "## 1. each file alone, then stacked: system < user (legacy) < user (XDG) < site"
conf system >"$XDG_CONFIG_DIRS/pip/pip.conf"
idx
conf user-legacy >"$HOME/.pip/pip.conf"
idx
conf user-xdg >"$XDG_CONFIG_HOME/pip/pip.conf"
idx
conf site >"$SITE"
idx
echo "## 2. a [install] section beats [global] in the same file"
printf '[install]\nindex-url = file://%s/site-install-section\n' "$W" >>"$SITE"
idx
conf site >"$SITE"
echo "## 3. PIP_CONFIG_FILE beats the site file, and drops the user files"
conf envfile >envfile.conf
PENV="PIP_CONFIG_FILE=$W/envfile.conf " idx
rm "$SITE"
PENV="PIP_CONFIG_FILE=$W/envfile.conf " idx
echo "## 4. PIP_INDEX_URL beats every file; the flag beats PIP_INDEX_URL"
PENV="PIP_INDEX_URL=file://$W/envvar " idx
PENV="PIP_INDEX_URL=file://$W/envvar " idx --index-url "file://$W/cli"
echo "## 5. --isolated drops the user files and PIP_* variables, not the system file"
PENV="PIP_INDEX_URL=file://$W/envvar " idx --isolated
echo "## 6. PIP_CONFIG_FILE=/dev/null drops every file"
PENV="PIP_CONFIG_FILE=/dev/null " idx
echo "## 7. a list (extra-index-url): files replace each other; the flag adds to the file's list"
printf '[global]\nindex-url = file://%s/base\nextra-index-url = file://%s/x-user\n' "$W" "$W" >"$XDG_CONFIG_HOME/pip/pip.conf"
printf '[global]\nextra-index-url = file://%s/x-site\n' "$W" >"$SITE"
idx
idx --extra-index-url "file://$W/x-cli"
PENV="PIP_EXTRA_INDEX_URL=file://$W/x-env " idx --extra-index-url "file://$W/x-cli"
echo "## 8. pip config debug lists what it read"
"$V/bin/python" -I -m pip config debug 2>&1 | sed "s#$W/##g; s#$V#<venv>#g"
