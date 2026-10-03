#!/usr/bin/env bash
# PROTOTYPE, never merged. What does each checker report on purba's Markdown
# today, and what does each fixer rewrite?
#
# The files are `git ls-files '*.md'`, passed by name, the way every task in
# `tasks.toml` passes its files. That reads `.template.md` and never `target/`.
#
# A fixer runs on a copy. Each file it changes is rendered by GitHub before and
# after, in `markdown` mode, the mode a file view uses. Identical HTML means the
# rewrite changed only the source; different HTML is listed for a person to
# judge as a repair or as damage.
set -u
cd "$(git rev-parse --show-toplevel)" || exit 2
# shellcheck source=prototype/lib.sh
. prototype/lib.sh
mapfile -t files < <(git ls-files '*.md' ':!:prototype/')
out=prototype/out
mkdir -p "${out}"

printf '## Checkers on %s files at %s\n\n' "${#files[@]}" "$(git rev-parse --short HEAD)"
printf '| checker | findings | files | rules (count) |\n|---|---:|---:|---|\n'
for tool in "${CHECKERS[@]}"; do
  "lint_${tool}" "${files[@]}" >"${out}/corpus.${tool}.tsv"
  n=$(wc -l <"${out}/corpus.${tool}.tsv")
  nf=$(cut -f1 "${out}/corpus.${tool}.tsv" | sort -u | grep -c .)
  rules=$(cut -f3 "${out}/corpus.${tool}.tsv" | sort | uniq -c | sort -rn |
    awk '{printf "%s%s %d", sep, $2, $1; sep=", "}')
  printf '| %s | %s | %s | %s |\n' "${tool}" "${n}" "${nf}" "${rules}"
done

render() { gh api markdown -f mode=markdown -f context=kalonji-tools/purba -F text=@"$1"; }

printf '\n## Fixers on the same files\n\n'
printf '| fixer | files rewritten | lines +/- | rendered HTML changed |\n|---|---:|---:|---|\n'
for tool in "${FIXERS[@]}"; do
  work=$(mktemp -d)
  git ls-files -z '*.md' ':!:prototype/' | xargs -0 cp --parents -t "${work}" .editorconfig
  (cd "${work}" && git init -q . && git add -A && git -c user.name=p -c user.email=p@p commit -qm base)
  (cd "${work}" && "fix_${tool}" "${files[@]}")
  changed=$(git -C "${work}" diff --name-only | grep -c .)
  stat=$(git -C "${work}" diff --numstat | awk '{a+=$1; d+=$2} END {printf "+%d/-%d", a, d}')
  html=()
  while read -r f; do
    [[ -n "${f}" ]] || continue
    if ! diff -q <(git -C "${work}" show "HEAD:${f}" | render /dev/stdin) \
      <(render "${work}/${f}") >/dev/null; then
      html+=("${f}")
      diff <(git -C "${work}" show "HEAD:${f}" | render /dev/stdin) <(render "${work}/${f}") \
        >"${out}/html.${tool}.$(tr / _ <<<"${f}").diff"
    fi
  done < <(git -C "${work}" diff --name-only)
  git -C "${work}" diff >"${out}/fix.${tool}.diff"
  printf '| %s | %s | %s | %s |\n' "${tool}" "${changed}" "${stat}" \
    "$([[ ${#html[@]} -eq 0 ]] && printf 'none' || printf '%s ' "${#html[@]}:" "${html[@]}")"
  rm -rf "${work}"
done
