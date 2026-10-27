#!/usr/bin/env bash
# shellcheck disable=SC2312,SC2249,SC2181,SC2164,SC2028 # throwaway prototype
# PROTOTYPE for "How does each part hand reporting what it finds, as it finds
# it?" (#664). Throwaway: this branch never merges.
#
#   mise exec -- bash proto-664/run.sh
#
# Builds the wheel of this branch, installs it in a fresh venv, and runs:
#   A  a test that crashes the process (os.abort, a segfault) as test 3 of 5,
#      under each way of printing (PURBA_PROTO_PRINT = vec | buffered | flushed)
#   B  `purba many.py | head -1` on 200 tests, slow (10 ms each) and fast, and
#      on 3000 fast tests (over 64 KiB of lines),
#      20 times under each answer to a refused write
#      (PURBA_PROTO_REFUSED = stop | quiet | ignore), and under today's vec.
set -u
ulimit -c 0

here=$(cd "$(dirname "$0")" && pwd)
top=$(git -C "${here}" rev-parse --show-toplevel)
work=${PROTO_WORK:-$(mktemp -d)}
runs=${PROTO_RUNS:-20}
mkdir -p "${work}/m"

cd "${top}"
rm -rf target/wheels
maturin build --quiet >/dev/null 2>&1 || {
  echo "maturin build failed"
  exit 2
}
python -m venv --clear "${work}/venv"
"${work}/venv/bin/python" -m pip install --quiet --force-reinstall --no-index \
  --find-links target/wheels purba
purba=${work}/venv/bin/purba

# ---- the modules -----------------------------------------------------------
crash_module() { # $1 = the statement test 3 runs
  {
    echo "import os, ctypes"
    for n in 1 2; do printf 'def test_%s():\n    assert True\n' "${n}"; done
    printf 'def test_3():\n    %s\n' "$1"
    for n in 4 5; do printf 'def test_%s():\n    assert True\n' "${n}"; done
  }
}
crash_module 'os.abort()' >"${work}/m/test_abort.py"
crash_module 'ctypes.string_at(0)' >"${work}/m/test_segv.py"

many_module() { # $1 = seconds each test sleeps, $2 = how many tests
  {
    echo "import os, time"
    echo "def _ran(n):"
    echo "    with open(os.environ['PROTO_LOG'], 'a') as log: log.write(n + '\n')"
    for n in $(seq -w 1 "${2:-200}"); do
      printf 'def test_%s():\n    _ran("%s")\n    time.sleep(%s)\n    assert True\n' \
        "${n}" "${n}" "$1"
    done
  }
}
many_module 0.01 >"${work}/m/test_slow.py"
many_module 0 >"${work}/m/test_fast.py"
many_module 0 3000 >"${work}/m/test_big.py"

# ---- A: a crash in test 3 of 5 --------------------------------------------
echo
echo "A. test 3 of 5 crashes the process: which lines reach the reader?"
echo
printf '%-9s %-6s %-5s %-5s %s\n' print crash to exit "lines the reader got"
for print in vec buffered flushed; do
  for crash in abort segv; do
    for to in file pipe; do
      out=${work}/a-${print}-${crash}-${to}.txt
      if [[ ${to} == file ]]; then
        (PURBA_PROTO_PRINT=${print} "${purba}" "${work}/m/test_${crash}.py" >"${out}") 2>/dev/null
        code=$?
      else
        PURBA_PROTO_PRINT=${print} "${purba}" "${work}/m/test_${crash}.py" 2>/dev/null |
          cat >"${out}"
        code=${PIPESTATUS[0]}
      fi
      lines=$(sed 's/.*:://; s/ .*//' "${out}" | paste -sd, -)
      printf '%-9s %-6s %-5s %-5s %s\n' "${print}" "${crash}" "${to}" "${code}" "${lines:-(none)}"
    done
  done
done

# ---- B: | head -1 -----------------------------------------------------------
echo
echo "B. purba test_<speed>.py | head -1, ${runs} runs each:" \
  "slow = 200 tests of 10 ms, fast = 200 tests, big = 3000 tests (over 64 KiB of lines)"
echo
printf '%-5s %-14s %-22s %-18s %s\n' speed answer "tests that ran" "exit codes" stderr
for speed in slow fast big; do
  for answer in vec stop quiet ignore; do
    ran_all=() codes=() errs=()
    for _ in $(seq "${runs}"); do
      log=${work}/b.log err=${work}/b.err
      : >"${log}"
      if [[ ${answer} == vec ]]; then
        PROTO_LOG=${log} PURBA_PROTO_PRINT=vec "${purba}" "${work}/m/test_${speed}.py" 2>"${err}" |
          head -1 >/dev/null
      else
        PROTO_LOG=${log} PURBA_PROTO_PRINT=flushed PURBA_PROTO_REFUSED=${answer} \
          "${purba}" "${work}/m/test_${speed}.py" 2>"${err}" | head -1 >/dev/null
      fi
      codes+=("${PIPESTATUS[0]}")
      ran_all+=("$(wc -l <"${log}")")
      errs+=("$(head -1 "${err}")")
    done
    ran=$(printf '%s\n' "${ran_all[@]}" | sort -n | uniq -c |
      awk '{printf "%s%s×%s", (NR>1?" ":""), $2, $1}')
    code=$(printf '%s\n' "${codes[@]}" | sort -n | uniq -c |
      awk '{printf "%s%s×%s", (NR>1?" ":""), $2, $1}')
    err=$(printf '%s\n' "${errs[@]}" | sort -u | sed 's/^$/(none)/' | paste -sd'|' -)
    label=${answer}
    [[ ${answer} == vec ]] && label="vec (today)"
    printf '%-5s %-14s %-22s %-18s %s\n' "${speed}" "${label}" "${ran}" "${code}" "${err}"
  done
done
echo
echo "work dir: ${work}"
