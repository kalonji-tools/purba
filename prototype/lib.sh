# PROTOTYPE, never merged. Sourced by the q1 scripts.
#
# One wrapper per candidate. A `lint_*` wrapper prints `path<TAB>line<TAB>rule`
# for each finding, with MD013 removed: purba's `.editorconfig` already turns
# line length off for Markdown. A `fix_*` wrapper rewrites the files it is given.
#
# Tools come through mise with the lock off, because none of them is in
# `mise.lock`. Node is precompiled: on NixOS mise otherwise builds it from
# source, and that needs `make`.
export MISE_LOCKED=0 MISE_NODE_COMPILE=0
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
cfg="${here}/config"

MDL='node@24 npm:markdownlint-cli2@0.23.3'
RUMDL='aqua:rvben/rumdl@0.2.78'
MADO='github:akiomik/mado@0.3.2'
PYMD='uv pipx:pymarkdownlnt@0.9.40'
MDFORMAT='uv pipx:mdformat@1.0.0'
PRETTIER='node@24 npm:prettier@3.9.9'
DPRINT='aqua:dprint/dprint@0.58.0'

CHECKERS=(markdownlint rumdl mado pymarkdown)
FIXERS=(markdownlint rumdl pymarkdown mdformat mdformat_gfm prettier dprint)

# Strip the worktree prefix that pymarkdown prints and drop MD013.
_norm() { sed -E "s#^${PWD}/##" | awk -F'\t' '$3 != "MD013"'; }

lint_markdownlint() {
  # shellcheck disable=SC2086
  mise x ${MDL} -- markdownlint-cli2 --config "${cfg}/.markdownlint-cli2.jsonc" "$@" 2>&1 |
    sed -nE 's/^([^:]+):([0-9]+)(:[0-9]+)? (error|warning) (MD[0-9]+)\/.*/\1\t\2\t\5/p' | _norm
}
lint_rumdl() {
  # shellcheck disable=SC2086
  mise x ${RUMDL} -- rumdl check --no-config -d MD013 "$@" 2>&1 |
    sed -nE 's/^([^:]+):([0-9]+):[0-9]+: \[(MD[0-9]+)\].*/\1\t\2\t\3/p' | _norm
}
lint_mado() {
  # shellcheck disable=SC2086
  mise x ${MADO} -- mado check "$@" 2>&1 |
    sed -nE 's/^([^:]+):([0-9]+):[0-9]+: (MD[0-9]+) .*/\1\t\2\t\3/p' | _norm
}
lint_pymarkdown() {
  # shellcheck disable=SC2086
  mise x ${PYMD} -- pymarkdown -d md013 scan "$@" 2>&1 |
    sed -nE 's/^([^:]+):([0-9]+):[0-9]+: ([A-Za-z]+[0-9]+): .*/\1\t\2\t\3/p' |
    awk -F'\t' -v OFS='\t' '{$3=toupper($3); print}' | _norm
}

fix_markdownlint() {
  # shellcheck disable=SC2086
  mise x ${MDL} -- markdownlint-cli2 --fix --config "${cfg}/.markdownlint-cli2.jsonc" "$@" >/dev/null 2>&1
}
fix_rumdl() {
  # shellcheck disable=SC2086
  mise x ${RUMDL} -- rumdl fmt --no-config -d MD013 "$@" >/dev/null 2>&1
}
fix_pymarkdown() {
  # shellcheck disable=SC2086
  mise x ${PYMD} -- pymarkdown -d md013 fix "$@" >/dev/null 2>&1
}
fix_mdformat() {
  # shellcheck disable=SC2086
  mise x ${MDFORMAT} -- mdformat "$@" >/dev/null 2>&1
}
fix_mdformat_gfm() {
  mise x uv -- uvx --quiet --from mdformat==1.0.0 --with mdformat-gfm mdformat "$@" >/dev/null 2>&1
}
fix_prettier() {
  # shellcheck disable=SC2086
  mise x ${PRETTIER} -- prettier --log-level silent --write "$@" >/dev/null 2>&1
}
fix_dprint() {
  # shellcheck disable=SC2086
  mise x ${DPRINT} -- dprint fmt --config "${cfg}/dprint.json" "$@" >/dev/null 2>&1
}
