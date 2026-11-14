#!/usr/bin/env bash
# RESEARCH (#694): runs the zygote stand-in REPEATS times over each module, and
# counts each test's lines. A line "raised: its process died by SIG..." is a crash.
set -u
cd "$(dirname "$0")" || exit 1
REPEATS=${REPEATS:-10}
python --version
sw_vers 2>/dev/null || uname -a
python make_mods.py
echo "## once: the Done-when, deaths and threads modules"
python zygote.py mods/test_done_when.py mods/test_deaths.py \
  mods/test_threads_left.py mods/test_thread_at_load.py
echo "## each call, ${REPEATS} runs"
: >raw.txt
for mod in mods/test_load_*.py mods/test_test_*.py; do
  for _ in $(seq "${REPEATS}" || true); do
    python zygote.py "${mod}" >>raw.txt || true
  done
done
grep -v '^-- ' raw.txt >lines.txt || true
sed -E 's/: .*//' lines.txt >kinds.txt || true
sort kinds.txt >sorted.txt || true
uniq -c sorted.txt || true
echo "## crash lines"
grep 'died by SIG' lines.txt >crashes.txt || true
wc -l <crashes.txt
sort crashes.txt >sorted.txt || true
uniq -c sorted.txt || true
