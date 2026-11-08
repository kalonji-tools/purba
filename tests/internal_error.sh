# shellcheck shell=bash
# The lines that purba writes to stderr for an internal error. Each value of the
# report comes from the venv of `purba` and from the OS, and not from purba's
# code.

# The two lines that ask for a report, for the `purba` at `$1`.
report_lines() {
  local purba=$1
  local python="${purba%/*}/python"
  local version cpython system machine os arch
  version=$("${python}" -c 'import importlib.metadata; print(importlib.metadata.version("purba"))')
  cpython=$("${python}" -c 'import platform, sys
print(platform.python_version() + sys.abiflags.replace("d", ""))')
  system=$(uname -s)
  machine=$(uname -m)
  # Rust names each OS in lower case, macOS as macos, and Apple's arm64 as aarch64:
  # https://doc.rust-lang.org/1.99.1/std/env/consts/constant.OS.html
  # https://doc.rust-lang.org/1.99.1/std/env/consts/constant.ARCH.html
  case ${system} in
    Darwin) os=macos ;;
    Linux) os=linux ;;
    *) os=${system} ;;
  esac
  case ${machine} in
    arm64) arch=aarch64 ;;
    *) arch=${machine} ;;
  esac
  printf '%s\n' \
    "purba: an internal error is a bug in purba ${version}, on CPython ${cpython}, ${os} ${arch}." \
    'purba: report it at https://github.com/kalonji-tools/purba/issues/new?template=defect.md'
}

# The last three lines of stderr after a panic that stops the run of the
# `purba` at `$1`, for the part `$2` and the message `$3`.
internal_error_lines() {
  local purba=$1 part=$2 message=$3
  printf '%s\n' "purba: internal error: ${part}: ${message}"
  report_lines "${purba}"
}
