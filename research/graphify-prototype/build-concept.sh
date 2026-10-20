#!/usr/bin/env bash
# PROTOTYPE (#602): build the concept graph for each fixture head on the
# person's subscription, under a scratch HOME.
set -uo pipefail
S=$(cd "$(dirname "$0")" && pwd)
LIBDIR=$(cut -d= -f2 "${S}/libdir")
cd "${S}" || exit 2
for p in "$@"; do
  out=${S}/graphs/${p}/concept
  mkdir -p "${out}"
  env -i HOME="${S}/home" \
    PATH="${S}/venv/bin:/etc/profiles/per-user/snregales/bin:/run/current-system/sw/bin" \
    LD_LIBRARY_PATH="${LIBDIR}" GRAPHIFY_NO_AUTO_REFRESH=1 GRAPHIFY_OUT="${out}" \
    "${S}/venv/bin/graphify" extract "${S}/corpus/${p}/decisions" --backend claude-cli \
    >"${S}/graphs/${p}/concept.log" 2>&1
  printf '%s\texit=%s\n' "${p}" "$?" >>"${S}/graphs/concept-times.tsv"
done
