#!/usr/bin/env bash
# RESEARCH (#694): runs the zygote stand-in REPEATS times over each module, and
# counts each test's lines. A line "raised: its process died by SIG..." is a crash.
# Three shapes of the same call, each REPEATS times:
#   zygote   purba -> fork zygote (loads) -> fork test   two forks deep
#   flat     purba loads -> fork test                    one fork deep
#   control  safety.py of #852: parent calls -> fork -> child calls
set -u
cd "$(dirname "$0")" || exit 1
REPEATS=${REPEATS:-10}
python --version
sw_vers 2>/dev/null || uname -a
python make_mods.py
echo "## once: the Done-when, deaths and threads modules"
python zygote.py mods/test_done_when.py mods/test_deaths.py \
  mods/test_threads_left.py mods/test_thread_at_load.py 2>/dev/null
: >zygote.lines
: >flat.lines
: >control.lines
: >warnings.txt
for mod in mods/test_load_*.py mods/test_test_*.py; do
  for _ in $(seq "${REPEATS}" || true); do
    python zygote.py "${mod}" >>zygote.lines 2>>warnings.txt || true
    python zygote.py --flat "${mod}" >>flat.lines 2>>warnings.txt || true
  done
done
for case in dns-both http-both proxies-both cf-prefs-both dns-child-only; do
  for _ in $(seq "${REPEATS}" || true); do
    python ../safety.py "${case}" >>control.lines 2>/dev/null || true
  done
done
for shape in zygote flat; do
  echo "## ${shape}, ${REPEATS} runs of each module"
  grep -v '^-- ' "${shape}.lines" >lines.txt || true
  sed -E -e 's/(raised: its process died by SIG[A-Z]+).*/\1/' \
    -e 's/(passed|raised|project error): .*/\1/' lines.txt >kinds.txt || true
  sort kinds.txt >sorted.txt || true
  uniq -c sorted.txt || true
done
echo "## control (#852 safety.py), ${REPEATS} runs of each case"
sed -E 's/"threads_at_fork": [0-9-]+, //' control.lines >kinds.txt || true
sort kinds.txt >sorted.txt || true
uniq -c sorted.txt || true
echo "## fork warnings: $(grep -c 'multi-threaded' warnings.txt || true)"
