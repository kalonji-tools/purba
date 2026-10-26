#!/usr/bin/env bash
# A throwaway probe: who opens fd 1 in a process whose shell closed it.
# shellcheck disable=SC2312
set -u

# Starts "$@" with fd 1 closed, and prints what fd 1 is while it runs.
show() {
  local label=$1
  shift
  "$@" >&- 2>/dev/null &
  local pid=$!
  sleep 2
  echo "-- ${label} (pid ${pid}, $(ps -o comm= -p "${pid}"))"
  lsof -a -p "${pid}" -d 1 2>&1 | tail -n +2
  for child in $(pgrep -P "${pid}"); do
    echo "   child ${child} $(ps -o comm= -p "${child}")"
    lsof -a -p "${child}" -d 1 2>&1 | tail -n +2
  done
  wait "${pid}"
}

nap='import time; time.sleep(4)'
py=$(command -v python)
echo "== python on PATH: ${py}"
"${py}" -VV
"${py}" -c 'import sys, sysconfig; print(sys.executable, sys._base_executable)
print("Py_GIL_DISABLED", sysconfig.get_config_var("Py_GIL_DISABLED"))
print("WITH_MIMALLOC", sysconfig.get_config_var("WITH_MIMALLOC"))'

echo "== interpreters on this runner"
ls -d /Library/Frameworks/Python*.framework/Versions/* 2>&1
ls /Library/Frameworks/Python*.framework/Versions/*/bin/python3* 2>&1
inner=$(find /Library/Frameworks/Python*.framework/Versions/*/Resources/Python.app/Contents/MacOS \
  -type f 2>/dev/null)
echo "${inner}"

echo "== fd 1 in each process"
show "sleep" /bin/sleep 4
show "python on PATH" "${py}" -c "${nap}"
show "python -I -S" "${py}" -I -S -c "${nap}"
show "PYTHONMALLOC=malloc" env PYTHONMALLOC=malloc "${py}" -c "${nap}"
show "PYTHON_GIL=1" env PYTHON_GIL=1 "${py}" -c "${nap}"
for exe in ${inner}; do
  show "app binary ${exe}" "${exe}" -c "${nap}"
done
for exe in /Library/Frameworks/Python*.framework/Versions/*/bin/python3.[0-9]*; do
  [[ -x "${exe}" ]] && show "${exe}" "${exe}" -c "${nap}"
done
[[ -x /usr/bin/python3 ]] && show "/usr/bin/python3" /usr/bin/python3 -c "${nap}"

echo "== the peer of fd 1"
"${py}" -c "${nap}" >&- 2>/dev/null &
pid=$!
sleep 2
lsof -a -p "${pid}" -d 1 2>&1
peer=$(lsof -a -p "${pid}" -d 1 -F n 2>/dev/null | sed -n 's/^n->//p')
echo "peer: ${peer}"
[[ -n "${peer}" ]] && sudo lsof -U 2>/dev/null | grep -i -- "${peer}"
echo "-- every unix socket that names syslog or log"
sudo lsof -U 2>/dev/null | grep -i -E 'syslog|logd|asl' | head -10
wait "${pid}"

echo "== libraries each interpreter loads"
for exe in ${inner}; do
  echo "-- ${exe}"
  otool -L "${exe}"
done
DYLD_PRINT_LIBRARIES=1 "${py}" -c pass 2>&1 >&- | sed 's/^/   /' | head -60
