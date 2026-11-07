#!/usr/bin/env bash
# RESEARCH (#852): does the build of the interpreter decide whether a fork crashes?
# For each interpreter: the macOS version and SDK its binary was linked against,
# then each crash case of safety.py, three times.
set -uo pipefail

here=$(cd "$(dirname "$0")" && pwd)
cases=(bare dns-both proxies-both cf-prefs-both dns-child-only)

pythons=()
for v in 3.12 3.14; do
  [[ -x /Library/Frameworks/Python.framework/Versions/${v}/bin/python${v} ]] &&
    pythons+=("/Library/Frameworks/Python.framework/Versions/${v}/bin/python${v}")
done
for v in 3.12 3.14; do
  uv python install "${v}" >/dev/null 2>&1 && pythons+=("$(uv python find "${v}")")
done
brew_prefix=$(brew --prefix)
for v in 3.12 3.14; do
  brew install "python@${v}" >/dev/null 2>&1
  [[ -x "${brew_prefix}/bin/python${v}" ]] && pythons+=("${brew_prefix}/bin/python${v}")
done

for py in "${pythons[@]}"; do
  real=$("${py}" -c 'import os, sys; print(os.path.realpath(sys.executable))')
  echo "## ${py}"
  echo "   real ${real}"
  "${py}" -c 'import sys; print("   version", sys.version.replace("\n", " "))'
  # The interpreter's code may live in a framework or a libpython, not the launcher.
  lib=$("${py}" -c 'import sysconfig; print(sysconfig.get_config_var("LDLIBRARY") or "")')
  libdir=$("${py}" -c 'import sysconfig; print(sysconfig.get_config_var("LIBDIR") or "")')
  for bin in "${real}" "${libdir}/${lib}"; do
    [[ -f "${bin}" ]] || continue
    echo "   ${bin}"
    otool -l "${bin}" | grep -A4 -E 'LC_BUILD_VERSION|LC_VERSION_MIN_MACOSX' |
      grep -E 'minos|sdk|version' | sed 's/^/     /'
  done
  for case in "${cases[@]}"; do
    results=()
    for _ in 1 2 3; do
      line=$("${py}" "${here}/safety.py" "${case}" 2>/dev/null | tail -n 1)
      results+=("$(sed -E 's/.*"child": "([^"]*)".*/\1/' <<<"${line:-parent failed}")")
    done
    printf '   %-16s %s\n' "${case}" "$(
      IFS=,
      echo "${results[*]}"
    )"
  done
done
