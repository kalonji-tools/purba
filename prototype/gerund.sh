#!/usr/bin/env bash
# Write out/g-<corpus>-<variant>.keys for the gerund rule, and score the
# disagreements against out/g-labels.tsv. Run corpus.sh first.
#
#   bash   scripts/check-records.sh at the head of #309, named in full below
#   lib    libwalk/ in its `ing-lead+u` variant
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
out=${here}/out
corpus=${here}/corpus
lib=${here}/libwalk/target/release/libwalk
[[ -x ${lib} ]] || cargo build --release --manifest-path "${here}/libwalk/Cargo.toml"

pick='/An -ing form is a technical noun/ { on = 1; next }'
pick+=' on && /^  [^ ]/ { print; next } on && !/^  / { on = 0 }'

bash_gate() { # dir
  local tree
  tree=$(mktemp -d)
  git -C "${here}/.." archive 8ba16680db9ee48bc3131f594d7c241243525f4b | tar -x -C "${tree}"
  (cd "${tree}" && git init -q && git add -A && git -c user.name=x -c user.email=x@x commit -qm x)
  (cd "${tree}" && env -u GIT_DIR -u GIT_WORK_TREE scripts/check-records.sh "$1" 2>&1 || true) |
    awk "${pick}" |
    sed "s|^ *$1/||; s|^ *||" | cut -d: -f1,2
  rm -rf "${tree}"
}

for name in records wild; do
  dir=${corpus}/${name}
  short=${name/records/rec}
  bash_gate "${dir}" | sort -u >"${out}/g-${short}-bash.keys"
  "${lib}" ing-lead+u "${dir}"/*.md | sed "s|^${dir}/||" | cut -d: -f1,2 | sort -u \
    >"${out}/g-${short}-lib.keys"
done

# Each disagreement, with its label: true, being, quote or false.
printf '| corpus | refused by | true | being | quoted | false | unlabelled |\n'
printf '|---|---|---|---|---|---|---|\n'
for short in rec wild; do
  for side in bash lib; do
    if [[ ${side} == bash ]]; then flag=-23; else flag=-13; fi
    comm "${flag}" "${out}/g-${short}-bash.keys" "${out}/g-${short}-lib.keys" \
      >"${out}/g-${short}-${side}-only.keys"
    row="| ${short} | ${side} only |"
    for label in true being quote false; do
      n=$(awk -F'\t' -v l="${label}" '$2 == l { print $1 }' "${out}/g-labels.tsv" |
        sort | comm -12 - "${out}/g-${short}-${side}-only.keys" | wc -l)
      row+=" ${n} |"
    done
    n=$(cut -f1 "${out}/g-labels.tsv" | sort |
      comm -13 - "${out}/g-${short}-${side}-only.keys" | wc -l)
    echo "${row} ${n} |"
  done
done
