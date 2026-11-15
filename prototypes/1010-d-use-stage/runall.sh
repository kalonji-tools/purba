#!/usr/bin/env bash
# Run every folder of this prototype (#1010, Q3) and save each output under runs/.
# PY is a Python 3.12 with flask==3.1.2 installed; see README.md.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
py="${PY:-${here}/.venv/bin/python}"
runs="${here}/runs"
mkdir -p "${runs}"
scratch="$(mktemp -d)" # each case's temporary database lands here; removed at the end
shop="${here}/undertest/shop"
flaskr="${here}/undertest/flaskr-tutorial"

# A line of the runner's output longer than 100 bytes, the width editorconfig-checker reads, is
# folded at a space, and a space at the end of a line is dropped.
fold100='
import sys
for line in (raw.rstrip() for raw in sys.stdin.read().splitlines()):
    indent = line[: len(line) - len(line.lstrip(" "))]
    words = line[len(indent) :].split(" ")
    cur = indent + words[0]
    for word in words[1:]:
        if len((cur + " " + word).encode()) <= 100:
            cur += " " + word
        else:
            print(cur.rstrip())
            cur = indent + "  " + word
    print(cur.rstrip())
'

# run <output name> <folder> <PYTHONPATH> [env …] -- <runner args …>
run() {
  local out="$1" dir="$2" path="$3"
  shift 3
  local envs=()
  while [[ "$1" != "--" ]]; do
    envs+=("$1")
    shift
  done
  shift
  local raw="${scratch}/${out}.raw"
  # the runner exits 1 when a case does not pass, which most of these runs show on purpose
  (cd "${dir}" && env TMPDIR="${scratch}" PYTHONPATH="${path}" "${envs[@]}" \
    "${py}" "${here}/run.py" "$@") >"${raw}" 2>&1 || true
  "${py}" -I -c "${fold100}" <"${raw}" >"${runs}/${out}.txt"
  local last
  last="$(tail -n 1 "${runs}/${out}.txt")"
  printf '%-34s %s\n' "${out}" "${last}"
}

run probes "${here}/probes" "${shop}" -- .
run probes_one-call "${here}/probes" "${shop}" FORM_FIXTURE=one-call -- .
run probes_far-alone "${here}/probes" "${shop}" -- test_use_far.py
for form in fixtures stages; do
  run "blog-${form}" "${here}/ports/blog-${form}" "${flaskr}" -- .
  run "blog-${form}_one-call" "${here}/ports/blog-${form}" "${flaskr}" FORM_FIXTURE=one-call -- .
  # the mutant: the test user's password hash in data.sql loses its last character, so the login
  # fails
  mutant="${scratch}/mutant-${form}"
  mkdir -p "${mutant}"
  cp "${here}/ports/blog-${form}/test_blog.py" "${mutant}/"
  sed '0,/acb2f/s//acb2/' "${here}/ports/blog-${form}/data.sql" >"${mutant}/data.sql"
  run "blog-${form}_mutant-login" "${mutant}" "${flaskr}" -- .
done

# replays: how often the stage's act (sign_up) runs in one run of the probe file
log="${scratch}/pipe.log"
(cd "${here}/probes" && env TMPDIR="${scratch}" PYTHONPATH="${shop}" PIPE_LOG="${log}" \
  "${py}" "${here}/run.py" test_use_stage.py >/dev/null 2>&1)
{
  echo "sign_up calls by name, over one run of probes/test_use_stage.py (each case is one process):"
  cut -d' ' -f2- "${log}" >"${log}.names"
  sort "${log}.names" >"${log}.sorted"
  uniq -c "${log}.sorted"
} >"${runs}/probes_replays.txt"

# time, best of three runs of each form
TMPDIR="${scratch}" PYTHONPATH="${flaskr}" "${py}" -I -c '
import subprocess, sys, time
here, py = sys.argv[1], sys.argv[2]
for form in ("fixtures", "stages"):
    times = []
    for _ in range(3):
        start = time.perf_counter()
        cwd = f"{here}/ports/blog-{form}"
        subprocess.run([py, here + "/run.py", "."], cwd=cwd, capture_output=True, check=False)
        times.append(time.perf_counter() - start)
    print(f"blog-{form:<9} best of 3: {min(times):.2f} s")
' "${here}" "${py}" >"${runs}/timing.txt"
cat "${runs}/timing.txt"

rm -rf "${scratch}" "${flaskr}/instance"
