#!/usr/bin/env bash
# RESEARCH (#694): Ctrl-C sends SIGINT to the whole foreground group: parent, zygote, test.
# usage: [MODS="mods/..."] ctrlc.sh N   (N interrupts, 0.3 s apart, the first 1 s in)
# test_stubborn.py swallows each interrupt for 8 s; test_slow.py does not.
cd "$(dirname "$0")" || exit 1
PY=${PY:-python}
n=${1:-1}
read -r -a mods <<<"${MODS:-mods/test_slow.py mods/test_deaths.py}"
setsid "${PY}" zygote.py "${mods[@]}" >ctrlc.out 2>&1 &
leader=$!
sleep 1
for _ in $(seq "${n}" || true); do
  kill -INT -- "-${leader}"
  sleep 0.3
done
start=${SECONDS}
wait "${leader}"
status=$?
end=${SECONDS}
cat ctrlc.out
echo "status ${status} (130 = died by SIGINT), waited $((end - start)) s after the last interrupt"
left=$(pgrep -g "${leader}" | wc -l || true)
echo "processes left in the group: ${left}"
