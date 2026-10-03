#!/usr/bin/env bash
# PROTOTYPE, never merged. Does a prek-level `exclude` keep `typos -w` out of CHANGELOG.md, with typos
# itself left unconfigured and no --force-exclude?
set -u
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE
src=$(git rev-parse --show-toplevel) || exit 2
work=$(mktemp -d)
git clone -q "${src}" "${work}/p" && cd "${work}/p" || exit 2
git checkout -q origin/main -B probe
git config user.name probe && git config user.email probe@example.invalid
mise trust -q mise.toml
split_hook() {
  awk '
    $0 == "id = \"typos\"" {in_typos = 1}
    in_typos && $0 == "name = \"typos\"" {print "name = \"typos -w\""; next}
    in_typos && $0 == "entry = \"mise x -- typos\"" {
      print "entry = \"mise x -- typos -w\""
      print "files = '\''\\.md$'\''"
      print "exclude = '\''^CHANGELOG\\.md$'\''"
      in_typos = 0; next}
    {print}' prek.toml >prek.new && mv prek.new prek.toml
  cat >>prek.toml <<'EOF'

[[repos.hooks]]
id = "typos-check"
name = "typos"
language = "system"
entry = "mise x -- typos"
exclude = '\.md$'
stages = ["pre-commit"]
EOF
}
split_hook
sed -n '/id = "typos"/,/stages/p' prek.toml
git add prek.toml && git commit -qm "chore: set the typos hooks (#219)" --no-verify
mise x -- prek install >/dev/null
msg="docs: probe the typos hooks (#219)"
attempt() { git commit -qm "${msg}" >"${work}/probe.log" 2>&1; printf '%s' "$?"; }

printf 'The recieve step.\n' >probe.md && git add probe.md
rc=$(attempt); printf 'md: exit %s, file `%s`, ' "${rc}" "$(cat probe.md)"
git add probe.md; printf 'second commit exit %s\n' "$(attempt)"

printf '#!/usr/bin/env bash\n# recieve the list\ntrue\n' >probe.sh && chmod +x probe.sh && git add probe.sh
rc=$(attempt); printf 'sh: exit %s, line `%s`, failed: %s\n' "${rc}" "$(sed -n 2p probe.sh)" \
  "$(sed -nE 's/^([^.]+)\.{3,}Failed$/\1/p' "${work}/probe.log" | paste -sd, -)"
git reset -q --hard HEAD

printf 'A recieve entry.\n' >CHANGELOG.md && git add CHANGELOG.md
rc=$(attempt); printf 'CHANGELOG.md: exit %s, file `%s`\n' "${rc}" "$(cat CHANGELOG.md)"

mise trust -q --untrust mise.toml
cd / && rm -rf "${work}"
