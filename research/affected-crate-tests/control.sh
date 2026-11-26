#!/usr/bin/env bash
# The control: apply one hand-made mutant of `errors`, run every Rust test of the
# workspace without stopping at the first failure, print the targets that
# failed, and revert the mutant.
# usage: control.sh <log file>   (run from the root of a purba checkout, under `mise exec --`)
set -u
log=$1
here=$(cd "$(dirname "$0")" && pwd)
LD_LIBRARY_PATH=$(python -c 'import sysconfig; print(sysconfig.get_config_var("LIBDIR"))')
export LD_LIBRARY_PATH
git apply "$here/control.patch" || exit 99
git diff --stat
cargo test --workspace --no-fail-fast >"$log" 2>&1
echo "exit: $?"
git apply -R "$here/control.patch" || exit 99
"$here/killers.sh" "$log"
