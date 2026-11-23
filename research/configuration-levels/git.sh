#!/usr/bin/env bash
# Probe: git's scopes system, global, local, worktree and command, and which wins.
# Usage: git.sh <empty-work-dir>
# GIT_CONFIG_SYSTEM and HOME point at files in the work dir, so /etc and ~ are never touched.
set -u
W=$1
mkdir -p "$W"
cd "$W" || exit 1
export HOME="$W/home" XDG_CONFIG_HOME="$W/home/.config" GIT_CONFIG_SYSTEM="$W/etc-gitconfig"
mkdir -p "$HOME" "$XDG_CONFIG_HOME/git"
unset GIT_CONFIG_GLOBAL GIT_CONFIG_NOSYSTEM GIT_CONFIG_PARAMETERS GIT_CONFIG_COUNT GIT_DIR GIT_WORK_TREE GIT_EDITOR VISUAL EDITOR
export GIT_AUTHOR_NAME=probe GIT_AUTHOR_EMAIL=probe@example.invalid GIT_COMMITTER_NAME=probe GIT_COMMITTER_EMAIL=probe@example.invalid
g() {
  echo "\$ git $*"
  git "$@" 2>&1 | sed "s#$W/##g"
  echo
}

git init -q repo
git -C repo commit -q --allow-empty -m init
git -C repo worktree add -q ../wt 2>/dev/null
git -C repo config extensions.worktreeConfig true

echo "## 1. one key at every scope: each later scope wins"
git config --file "$GIT_CONFIG_SYSTEM" probe.key system
git config --file "$XDG_CONFIG_HOME/git/config" probe.key global-xdg
git config --file "$HOME/.gitconfig" probe.key global-home
git -C repo config --local probe.key local
git -C wt config --worktree probe.key worktree
g -C repo config --show-scope --show-origin --get-all probe.key
g -C repo config --get probe.key
g -C wt config --get probe.key
g -C wt -c probe.key=command config --get probe.key
GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=probe.key GIT_CONFIG_VALUE_0=env-count g -C wt config --get probe.key
GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=probe.key GIT_CONFIG_VALUE_0=env-count g -C wt -c probe.key=command config --show-scope --get-all probe.key

echo "## 2. GIT_CONFIG_NOSYSTEM and GIT_CONFIG_GLOBAL drop a scope"
GIT_CONFIG_NOSYSTEM=1 g -C repo config --show-scope --get-all probe.key
GIT_CONFIG_GLOBAL=/dev/null g -C repo config --show-scope --get-all probe.key

echo "## 3. a multi-valued key: every scope adds, and an empty value resets the list"
git config --file "$GIT_CONFIG_SYSTEM" credential.helper sys-helper
git config --file "$HOME/.gitconfig" credential.helper home-helper
git -C repo config --local --add credential.helper ""
git -C repo config --local --add credential.helper local-helper
g -C repo config --show-scope --get-all credential.helper
echo "   (git credential fill would run only local-helper, because the empty value resets the list)"

echo "## 4. safe.directory: read from system, global and command, ignored in local"
# GIT_TEST_ASSUME_DIFFERENT_OWNER is git's own test switch: git then treats the repository
# as owned by another user, which is the case safe.directory exists for.
export GIT_TEST_ASSUME_DIFFERENT_OWNER=1
g -C repo status --short
git -C repo config --local safe.directory '*' 2>/dev/null || git config --file repo/.git/config safe.directory '*'
echo "   with safe.directory=* in the local file:"
g -C repo status --short
echo "   with safe.directory=* on the command line:"
g -c safe.directory='*' -C repo status --short
echo "   with safe.directory=* in the global file:"
git config --file "$HOME/.gitconfig" safe.directory '*'
g -C repo status --short
git config --file "$HOME/.gitconfig" --unset safe.directory
unset GIT_TEST_ASSUME_DIFFERENT_OWNER

echo "## 5. an environment variable beats the key: GIT_EDITOR over core.editor"
git -C repo config --local core.editor from-config
g -C repo var GIT_EDITOR
GIT_EDITOR=from-env g -C repo var GIT_EDITOR
GIT_EDITOR=from-env g -C repo -c core.editor=from-command var GIT_EDITOR
