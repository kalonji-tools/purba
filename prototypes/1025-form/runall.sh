#!/usr/bin/env bash
# Run the probes, the examples and the regression targets, and save each output under runs/.
# PY is a Python 3.12 with flask==3.1.2 and loguru==0.8.0 installed; see README.md.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
py="${PY:-${here}/.venv/bin/python}"
runs="${here}/runs"
mkdir -p "${runs}"
scratch="$(mktemp -d)" # each run's act log and summary, and each case's temporary files
undertest="${here}/undertest"
pd="${here}/../1010-d-use-stage"
flaskr="${pd}/undertest/flaskr-tutorial"
pa="${PA:-${here}/../../../purba.proto-1010-a-fork-tree/prototypes/1010-a-fork-tree}"

# A line of the runner's output longer than 100 bytes is folded at a space.
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
  : >"${scratch}/${out}.acts"
  # the runner exits 1 when a case does not pass, which most of these runs show on purpose
  (cd "${dir}" && env TMPDIR="${scratch}" PYTHONPATH="${path}" PYTHONDONTWRITEBYTECODE=1 \
    FORM_ACTS="${scratch}/${out}.acts" FORM_SUMMARY="${scratch}/${out}.json" "${envs[@]}" \
    "${py}" "${here}/run.py" "$@") >"${raw}" 2>&1 || true
  "${py}" -I -c "${fold100}" <"${raw}" >"${runs}/${out}.txt"
  printf '%-30s %s\n' "${out}" "$(tail -n 1 "${runs}/${out}.txt")"
}

# verify <output name> <probe file …>: each probe's docstring against the run's summary
verify() {
  local out="$1"
  shift
  (cd "${here}/probes" && "${py}" -I "${here}/verify.py" "${scratch}/${out}.json" "$@") \
    >"${runs}/${out}_verify.txt"
  printf '%-30s %s\n' "${out}_verify" "$(tail -n 1 "${runs}/${out}_verify.txt")"
}

run probes "${here}/probes" "${undertest}" -- .
verify probes $(cd "${here}/probes" && ls test_*.py)
run probes_far-alone "${here}/probes" "${undertest}" -- test_far.py
verify probes_far-alone test_far.py test_far_home.py

# the seed of random (#1015): the same --seed repeats each case's draws, and a picked seed differs
run seed_1015-a "${here}/probes" "${undertest}" -- --seed 1015 test_1015_seed.py
run seed_1015-b "${here}/probes" "${undertest}" -- --seed 1015 test_1015_seed.py
run seed_picked "${here}/probes" "${undertest}" -- test_1015_seed.py
{
  echo "the draws of --seed 1015, run twice (no diff means the same values):"
  diff <(grep 'reading' "${runs}/seed_1015-a.txt") <(grep 'reading' "${runs}/seed_1015-b.txt")
  echo "the draws of --seed 1015 against a seed the runner picked:"
  diff <(grep 'reading' "${runs}/seed_1015-a.txt") <(grep 'reading' "${runs}/seed_picked.txt")
} >"${runs}/seed_compare.txt"

# a control: six of the new probe files under the stage-as-input prototype's runner
: >"${scratch}/control.acts"
(cd "${here}/probes" && env TMPDIR="${scratch}" PYTHONPATH="${undertest}" PYTHONDONTWRITEBYTECODE=1 \
  FORM_ACTS="${scratch}/control.acts" "${py}" "${pd}/run.py" test_tree.py test_blocked.py \
  test_before_each.py test_watch.py test_reading_assert.py test_helper_lines.py) \
  >"${scratch}/control.raw" 2>&1 || true
"${py}" -I -c "${fold100}" <"${scratch}/control.raw" >"${runs}/control_previous-runner.txt"
printf '%-30s %s\n' control_previous-runner "$(tail -n 1 "${runs}/control_previous-runner.txt")"

run examples "${here}/examples/tests" "${here}/examples/src" -- .
run examples_blocked "${here}/examples/blocked" "${here}/examples/src" -- .

# How often each act of the shop ran, over one run of probes/test_use_stage.py: each `sign_up ann`
# is one act, under the fork tree.
log="${scratch}/pipe.log"
(cd "${here}/probes" && env TMPDIR="${scratch}" PYTHONPATH="${undertest}" PIPE_LOG="${log}" \
  PYTHONDONTWRITEBYTECODE=1 FORM_ACTS="${scratch}/use.acts" "${py}" "${here}/run.py" \
  test_use_stage.py >/dev/null 2>&1)
{
  echo "acts of the shop by name, over one run of probes/test_use_stage.py:"
  cut -d' ' -f2- "${log}" | sort | uniq -c
} >"${runs}/probes_acts.txt"

# regression: the flaskr ports of the stage-as-input prototype, and the chains of the fork-tree one
for form in fixtures stages; do
  run "blog-${form}" "${pd}/ports/blog-${form}" "${flaskr}" -- .
  mutant="${scratch}/mutant-${form}"
  mkdir -p "${mutant}"
  cp "${pd}/ports/blog-${form}/test_blog.py" "${mutant}/"
  # the test user's password hash loses its last character, so the login fails
  sed '0,/acb2f/s//acb2/' "${pd}/ports/blog-${form}/data.sql" >"${mutant}/data.sql"
  run "blog-${form}_mutant-login" "${mutant}" "${flaskr}" -- .
done
if [[ -d "${pa}/chains" ]]; then
  run chains "${pa}/chains" "${pa}/undertest_lib" -- .
else
  echo "chains: skipped, ${pa}/chains is missing"
fi

rm -rf "${scratch}" "${flaskr}/instance"
