#!/usr/bin/env bash
# A throwaway probe: does the bundle's missing executable open the syslog socket?
# shellcheck disable=SC2312
set -u

nap='import time; time.sleep(4)'
py=$(command -v python)
"${py}" -VV

# Starts "$@" with fd 1 closed, and prints fd 1 while it runs.
show() {
  local label=$1
  shift
  "$@" >&- 2>/dev/null &
  local pid=$!
  sleep 2
  echo "-- ${label}: $(lsof -a -p "${pid}" -d 1 2>&1 | tail -n +2)"
  wait "${pid}"
}

for app in /Library/Frameworks/Python*.framework/Versions/3.14/Resources/Python.app; do
  echo "== ${app}"
  plutil -p "${app}/Contents/Info.plist" | grep -E 'CFBundle(Executable|Identifier|Name)"'
  ls -la "${app}/Contents/MacOS/"
done

app=/Library/Frameworks/PythonT.framework/Versions/3.14/Resources/Python.app
macos="${app}/Contents/MacOS"
want=$(plutil -extract CFBundleExecutable raw "${app}/Contents/Info.plist")
echo "== CFBundleExecutable of PythonT's bundle: ${want}"

show "before" "${py}" -c "${nap}"
if [[ ! -e "${macos}/${want}" ]]; then
  sudo ln -s PythonT "${macos}/${want}"
  echo "linked ${macos}/${want} -> PythonT"
  show "with the executable the plist names" "${py}" -c "${nap}"
  start=$(date '+%Y-%m-%d %H:%M:%S')
  "${py}" -c pass >&- 2>/dev/null
  sleep 2
  echo "-- logged after the link:"
  log show --start "${start}" --info --debug --style compact \
    --predicate 'process == "PythonT"' 2>&1 | tail -n +2 | head -10
  sudo rm "${macos}/${want}"
  show "after removing the link" "${py}" -c "${nap}"
fi
