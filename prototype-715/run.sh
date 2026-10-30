#!/usr/bin/env bash
# PROTOTYPE (#715), never merge. Builds a wheel of main and a wheel of this
# branch, then runs each case under each catch mode, and prints stdout, the
# first lines of stderr, and the exit code.
#   PURBA_CATCH: none (today) | dispatch | execution | hook | record
#   PURBA_FAULT: load-own-call | hook-back
#   in a test:   `assert "purba:panic"` makes the hook panic;
#                a test named *_panics_before_call panics in execution.
set -u
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/.." && pwd)
out=${1:-$root/target/proto-715}
build() { # build <name> <source dir> <target dir>
  (cd "$2" && CARGO_TARGET_DIR="$3" mise exec -- maturin build --quiet --out "$out/wheel-$1")
  mise exec -- python -m venv --clear "$out/venv-$1"
  "$out/venv-$1/bin/pip" install -q --force-reinstall --no-index --find-links "$out/wheel-$1" purba
}
if [[ -z ${SKIP_BUILD:-} ]]; then
  rm -rf "$out/main-src" && mkdir -p "$out/main-src"
  git -C "$root" archive origin/main | tar -x -C "$out/main-src"
  build main "$out/main-src" "$out/main-target"
  build proto "$root" "$root/target"
fi
show() { # show <label> <venv> <file> [env...]
  local label=$1 venv=$2 file=$3
  shift 3
  echo "### $label"
  env "$@" "$out/venv-$venv/bin/purba" "$file" 2>"$out/stderr"
  echo "exit=$?"
  sed 's/^/  stderr| /' "$out/stderr" | head -6
}
cd "$here/q1"
show "main" main test_q1.py
for mode in dispatch execution hook record; do
  show "PURBA_CATCH=$mode" proto test_q1.py PURBA_CATCH=$mode
done
cd "$here/q3"
for file in test_body_raises.py test_return_outside.py test_binds_hook.py test_pops_name.py test_forged_index.py test_rebinds_in_test.py; do
  show "main $file" main "$file"
  show "proto $file" proto "$file" PURBA_CATCH=execution
done
show "proto test_rebinds_in_test.py PURBA_FAULT=hook-back" proto test_rebinds_in_test.py PURBA_CATCH=execution PURBA_FAULT=hook-back
show "proto test_plain.py PURBA_FAULT=load-own-call" proto test_plain.py PURBA_CATCH=execution PURBA_FAULT=load-own-call
