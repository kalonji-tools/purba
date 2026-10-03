#!/usr/bin/env bash
# PROTOTYPE, never merged. Where should `typos -w` write?
#
# Every arm runs in a scratch clone of this branch, with purba's real
# `prek.toml` and `mise.toml`; only the typos hook entry varies.
#
#   A  today: `typos`, no -w
#   B  `typos -w` on every file
#   C  `typos -w` on Markdown, `typos` on the rest
#
# Scenarios, each a commit attempt:
#   md        a misspelling in a Markdown file
#   sh        a misspelling in a shell comment
#   ident     a misspelled function, defined in a committed file and called
#             from the staged one. Only reachable when the tree is already red,
#             as after a typos release that learns a new word.
#   partial   a fixable misspelling in the staged half of a file whose other
#             half is unstaged
#   ambig     a word with two corrections
set -u
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE
src=$(git rev-parse --show-toplevel) || exit 2

entry_A='entry = "mise x -- typos"'
entry_B='entry = "mise x -- typos -w"'

setup() { # $1 = arm
  work=$(mktemp -d)
  git clone -q "${src}" "${work}/p" && cd "${work}/p" || exit 2
  git config user.name probe && git config user.email probe@example.invalid
  mise trust -q mise.toml
  case "$1" in
    A) ;;
    B) sed -i "s|^${entry_A}\$|${entry_B}|" prek.toml ;;
    C)
      sed -i "s|^${entry_A}\$|${entry_B}\nfiles = '\\\\.md\$'|" prek.toml
      cat >>prek.toml <<'EOF'

[[repos.hooks]]
id = "typos-check"
name = "typos"
language = "system"
entry = "mise x -- typos"
exclude = '\.md$'
stages = ["pre-commit"]
EOF
      ;;
  esac
  git add prek.toml && git commit -qm "chore: set the typos arm (#219)" --no-verify
  mise x -- prek install -q >/dev/null 2>&1 || mise x -- prek install >/dev/null
  [[ -x .git/hooks/pre-commit ]] || { echo 'no hook installed'; exit 2; }
}
teardown() { mise trust -q --untrust mise.toml; cd / && rm -rf "${work}"; }

attempt() { # commit; print the exit and the hooks that failed
  local log rc
  log=$(git commit -qm "docs: probe the typos hook (#219)" 2>&1)
  rc=$?
  printf "%s%s" "${rc}" "$(sed -nE "s/^([^.]+)\.{3,}Failed$/ (\1)/p" <<<"${log}" | paste -sd, -)"
}

printf '| arm | scenario | commit exit | file afterwards | then |\n|---|---|---:|---|---|\n'
for arm in A B C; do
  # md
  setup "${arm}"
  printf 'The recieve step.\n' >probe.md && git add probe.md
  rc=$(attempt)
  printf '| %s | md | %s | `%s` | %s |\n' "${arm}" "${rc}" "$(cat probe.md)" \
    "$(git add probe.md && attempt | sed 's/^/second commit exit /')"
  teardown

  # sh
  setup "${arm}"
  printf '#!/usr/bin/env bash\n# recieve the list\ntrue\n' >probe.sh && git add probe.sh
  rc=$(attempt)
  printf '| %s | sh comment | %s | `%s` | |\n' "${arm}" "${rc}" "$(sed -n 2p probe.sh)"
  teardown

  # ident: the definition is committed past the hook; the caller is staged.
  setup "${arm}"
  printf 'recieve_all() { echo ok; }\n' >lib.sh
  git add lib.sh && git commit -qm base --no-verify
  printf '#!/usr/bin/env bash\n. ./lib.sh\nrecieve_all\n' >main.sh && git add main.sh
  rc=$(attempt)
  printf '| %s | ident | %s | `%s` | runs: %s |\n' "${arm}" "${rc}" "$(sed -n 3p main.sh)" \
    "$(bash main.sh 2>&1 | head -1)"
  teardown

  # partial
  setup "${arm}"
  printf 'first line\n\nsecond line\n' >half.md && git add half.md && git commit -qm base --no-verify
  printf 'The recieve step.\n\nsecond line\n' >half.md && git add half.md
  printf 'The recieve step.\n\nsecond line, unstaged edit\n' >half.md
  rc=$(attempt)
  printf '| %s | partial | %s | `%s` | staged: `%s` |\n' "${arm}" "${rc}" "$(tr '\n' ' ' <half.md)" \
    "$(git show :half.md | head -1)"
  teardown

  # ambig
  setup "${arm}"
  printf 'The wich step.\n' >amb.md && git add amb.md
  rc=$(attempt)
  printf '| %s | ambig | %s | `%s` | |\n' "${arm}" "${rc}" "$(cat amb.md)"
  teardown
done

# An exclusion must survive prek naming the file. typos checks a path it is
# given even when its config excludes it, unless told otherwise.
printf '\n### Does a typos exclusion hold when prek names the file?\n\n'
work=$(mktemp -d) && cd "${work}" || exit 2
printf '[files]\nextend-exclude = ["CHANGELOG.md"]\n' >_typos.toml
printf 'A recieve entry.\n' >CHANGELOG.md
for flags in '' '--force-exclude'; do
  cp CHANGELOG.md before.md
  # shellcheck disable=SC2086
  (cd "${src}" && mise x -- typos -w ${flags} --config "${work}/_typos.toml" "${work}/CHANGELOG.md") >/dev/null 2>&1
  printf -- '- `typos -w %s CHANGELOG.md` with `extend-exclude`: file now `%s`\n' "${flags}" "$(cat CHANGELOG.md)"
  cp before.md CHANGELOG.md
done
cd / && rm -rf "${work}"
