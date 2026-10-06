#!/usr/bin/env bash
# Write out/<corpus>-<variant>.keys: the `file:line` keys each variant refuses,
# on the lines the bash gate reads. A table row or a heading is dropped.
#
#   walk    scripts/check-records.sh at the head of #309, named in full below
#   weir    weir/PerfectTense.weir, through `harper-cli` 2.3.0
#   union   walk or weir
#   lib-*   libwalk/, with harper-core 2.3.0 as a library
#
# Needs harper-cli 2.3.0 on PATH: `mise exec harper-cli@2.3.0 -- ./keys.sh`.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
out=${here}/out
corpus=${here}/corpus
lib=${here}/libwalk/target/release/libwalk
[[ -x ${lib} ]] || cargo build --release --manifest-path "${here}/libwalk/Cargo.toml"

in_scope() { # dir; reads file:line keys on stdin
  while IFS= read -r key; do
    line=$(sed -n "${key#*:}p" "$1/${key%%:*}" | sed 's/^[[:space:]]*//')
    if [[ ${line} != '|'* && ${line} != '#'* ]]; then echo "${key}"; fi
  done
}

walk() { # dir
  local tree
  tree=$(mktemp -d)
  git -C "${here}/.." archive 8ba16680db9ee48bc3131f594d7c241243525f4b | tar -x -C "${tree}"
  (cd "${tree}" && git init -q && git add -A && git -c user.name=x -c user.email=x@x commit -qm x)
  (cd "${tree}" && env -u GIT_DIR -u GIT_WORK_TREE scripts/check-records.sh "$1" 2>&1 || true) |
    awk '/simple tenses/ { on = 1; next } on && /^  [^ ]/ { print; next } on && !/^  / { on = 0 }' |
    sed "s|^ *$1/||; s|^ *||" | cut -d: -f1,2
  rm -rf "${tree}"
}

rm -f "${out}/PerfectTense.weirpack"
(cd "${here}/weir" && zip -q "${out}/PerfectTense.weirpack" manifest.json PerfectTense.weir)

for name in records wild; do
  dir=${corpus}/${name}
  short=${name/records/rec}
  walk "${dir}" | sort -u >"${out}/${short}-walk.keys"
  # It exits 1 when it finds a lint, as a gate does.
  { harper-cli lint --no-color --format json --weirpacks "${out}/PerfectTense.weirpack" \
    --only PerfectTense "${dir}"/*.md 2>/dev/null || true; } |
    jq -r '.[] | .file as $f | .lints[] | "\($f):\(.line)"' |
    sort -u | in_scope "${dir}" | sort -u >"${out}/${short}-weir.keys"
  sort -u "${out}/${short}-walk.keys" "${out}/${short}-weir.keys" >"${out}/${short}-union.keys"
  for v in det np det+v np+v; do
    "${lib}" "${v}" "${dir}"/*.md | sed "s|^${dir}/||" | cut -d: -f1,2 | sort -u \
      >"${out}/${short}-lib-${v}.keys"
  done
done
