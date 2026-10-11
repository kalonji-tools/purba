#!/usr/bin/env bash
# PROTOTYPE for issue 488. Never merge.
#
# Rule 2, the derived list: build the way Build does, collect the tracked paths
# the build read, and refuse one the build-reads file does not mark `build`.
# It needs a build, so in CI it runs inside Build, and only when the detector
# already said run.
#
#   derive.sh depinfo <build-reads file> <target dir>
#   derive.sh strace  <build-reads file> <target dir>
#
# depinfo: the paths Cargo's dep-info files name after `cargo test
#   --all-targets --no-run`. Portable, sees only what rustc and build scripts
#   declare.
# strace: every tracked path that `cargo test --all-targets --no-run` and
#   `maturin build` open, as "Which tracked paths do the jobs of build.yml
#   read?" traced them. Linux only.
#
# A first attempt read maturin's half from `maturin sdist`. It named every
# tracked file, because the sdist takes Cargo's package list, and Build never
# builds an sdist.
#
# Run at the top of a checkout with cargo and maturin on PATH; never starts mise.
# Prints `pass (<n> read)` or `refuse: <paths>`.
set -euo pipefail
here=$(dirname "$(realpath "$0")")
mode=$1 list=$2 target=$3
top=$(pwd -P)
out=$(mktemp -d)
trap 'rm -rf "${out}"' EXIT
git ls-files | sort >"${out}/tracked"

case "${mode}" in
depinfo)
  CARGO_TARGET_DIR="${target}" cargo test --all-targets --no-run --quiet >"${out}/log" 2>&1 ||
    { echo "cargo failed:"; tail -5 "${out}/log"; }
  # A dep-info line is `target: dep dep ...`, with a space in a path written
  # `\ `. Cargo writes a path inside the workspace relative to its root.
  find "${target}" -name '*.d' -print0 |
    xargs -0 -r cat | sed 's/\\ /\x00/g' | tr ' ' '\n' | tr '\0' ' ' |
    sed "s/:$//; s|^${top}/||" | sort -u >"${out}/read"
  ;;
strace)
  : >"${out}/all"
  for cmd in "cargo test --all-targets --no-run --quiet" "maturin build"; do
    # shellcheck disable=SC2086
    CARGO_TARGET_DIR="${target}" strace -f -qq -e trace=open,openat,execve -e status=successful \
      -o "${out}/trace" ${cmd} >>"${out}/log" 2>&1 ||
      { echo "${cmd} failed:"; tail -5 "${out}/log"; }
    cat "${out}/trace" >>"${out}/all"
  done
  # A relative path is relative to the working directory, which every traced
  # process here starts in.
  sed -nE 's/.*(open|openat|execve)\((AT_FDCWD, )?"([^"]*)".*/\3/p' "${out}/all" |
    sed "s|^${top}/||; s|^\./||" | sort -u >"${out}/read"
  ;;
*)
  echo "usage: derive.sh depinfo|strace <build-reads file> <target dir>" >&2
  exit 2
  ;;
esac

# Cargo writes `src/../docs/x.md` for an `include_str!("../docs/x.md")`, so each
# path is normalised before it is compared. Only a tracked path can be listed,
# so an untracked one is left out.
{ grep -v "^$" "${out}/read" || true; } | xargs -r -d "\n" realpath -m --relative-to=. -- | sort -u >"${out}/normal"
comm -12 "${out}/normal" "${out}/tracked" >"${out}/reads"
unlisted=$("${here}/attr.sh" "${list}" <"${out}/reads" | { grep -v '^set ' || true; } | cut -d' ' -f2-)
if [[ -z "${unlisted}" ]]; then
  echo "pass ($(wc -l <"${out}/reads") read: $(tr '\n' ' ' <"${out}/reads"))"
else
  echo "refuse: $(tr '\n' ' ' <<<"${unlisted}")"
fi
