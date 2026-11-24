#!/usr/bin/env bash
# Mercurial's revsets on the probe suite. Each test is one commit: its description holds the
# id and the tags, and it adds one file under the test's module.
#
#   hg-run.sh <empty dir>        run it inside: nix-shell -p mercurial
set -uo pipefail
# nix-shell may export every variable; a long query would then overflow the environment.
set +a

dir=${1:?an empty directory}
cd "${dir}" || exit 2
export HGPLAIN=1 HGRCPATH=/dev/null HGUSER=probe
hg init -q suite
cd suite || exit 2
while read -r module name tags; do
  mkdir -p "${module}"
  printf '%s\n' "${name}" >"${module}/${name}"
  hg add -q "${module}/${name}"
  hg commit -q -m "${module}::${name} ${tags}"
done <<'EOF'
math add -
math add_slow slow
store query db
store migrate slow db
store load slow flaky
store save slow
EOF

ask() {
  printf -- "--- hg log %s\n" "$*"
  hg log -T '{desc|firstline}\n' "$@" 2>&1
  printf 'exit %s\n' "$?"
}
tree() {
  printf -- '--- hg debugrevspec -p parsed %s\n' "$1"
  hg debugrevspec -p parsed "$1" 2>&1 | grep -v '^[0-9]*$'
}

echo "== $(hg --version | head -1)"
echo "== the example: the slow tests of store, but not the flaky ones"
ask -r 'file("glob:store/*") and desc(slow) and not desc(flaky)'
ask -r 'file("glob:store/*") & desc(slow) - desc(flaky)'
echo "== precedence"
tree 'desc(db) or desc(slow) and not desc(db)'
ask -r 'desc(db) or desc(slow) and not desc(db)'
tree 'desc(db) | desc(slow) - desc(db)'
echo "== aliases, given as config"
ask --config 'revsetalias.slow_store=file("glob:store/*") and desc(slow)' \
  --config 'revsetalias.stable($1)=$1 and not desc(flaky)' -r 'stable(slow_store)'
echo "== an id of purba as a query"
tree 'store::migrate'
ask -r 'store::migrate'
tree 'tests/api'
echo "== errors at a position"
ask -r 'desc(slow) and'
ask -r 'desc(slow) and (desc(db)'
ask -r 'tag(slow)'
ask -r 'desk(slow)'
echo "== nesting: N pairs of parentheses around desc(slow)"
for n in 10 100 200 300 500 1000 10000; do
  q=$(awk -v n="${n}" 'BEGIN{s="";for(i=0;i<n;i++)s=s "(";s=s "desc(slow)";for(i=0;i<n;i++)s=s ")";printf "%s",s}')
  hg log -T "{rev}\n" -r "${q}" >"${dir}/nest.out" 2>&1
  code=$?
  printf "N=%s exit=%s %s\n" "${n}" "${code}" "$(grep -v "^[0-9]*$" "${dir}/nest.out" | tail -1 | cut -c1-120)"
done
