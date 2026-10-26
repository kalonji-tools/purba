#!/usr/bin/env bash
# A throwaway probe: why the free-threaded Python opens the syslog socket.
# shellcheck disable=SC2312
set -u

nap='import time; time.sleep(4)'
py=$(command -v python)
"${py}" -VV

# Starts "$@" with fd 1 closed, keeps its stderr, and prints its fds while it runs.
show() {
  local label=$1
  shift
  local err
  err=$(mktemp)
  "$@" >&- 2>"${err}" &
  local pid=$!
  sleep 2
  echo "-- ${label} (pid ${pid}, $(ps -o comm= -p "${pid}"))"
  lsof -a -p "${pid}" -d 0-9 2>&1 | tail -n +2
  wait "${pid}"
  echo "   exit $?, stderr: $(tr '\n' '|' <"${err}")"
}

echo "== with fd 1 closed"
show "python" "${py}" -c "${nap}"
show "MallocNanoZone=0" env MallocNanoZone=0 "${py}" -c "${nap}"
show "MallocNanoZone=1" env MallocNanoZone=1 "${py}" -c "${nap}"
show "PYTHONMALLOC=malloc" env PYTHONMALLOC=malloc "${py}" -c "${nap}"
show "PYTHONMALLOC=mimalloc" env PYTHONMALLOC=mimalloc "${py}" -c "${nap}"

echo "== with fd 1 open on /dev/null: where the socket lands"
"${py}" -c "${nap}" >/dev/null 2>&1 &
pid=$!
sleep 2
lsof -a -p "${pid}" -d 0-9 2>&1
wait "${pid}"

echo "== what the process logged"
start=$(date '+%Y-%m-%d %H:%M:%S')
"${py}" -c pass >&- 2>/dev/null
sleep 2
log show --start "${start}" --info --debug --style compact \
  --predicate 'process == "PythonT" OR process == "Python"' 2>&1 | head -40
