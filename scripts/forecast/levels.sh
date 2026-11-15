#!/usr/bin/env bash
# levels.sh OUT UNTIL PURBA_TREE
# Maps what forecast.sh, probe.sh and github.sh wrote to a level from 1 to 3 for blast area
# and the seven risks, by the scales of
# docs/decisions/a-crate-joins-when-its-relief-exceeds-its-pain.md.
# "-" means the script gives no level.
# OUT/crates.tsv holds "crate<TAB>version<TAB>parts", and OUT/github/repos.tsv
# "crate<TAB>owner/repo", or "crate<TAB>-" for a repository that is not on GitHub.
# UNTIL cuts the crates.io versions. Prints "crate version blast upkeep maintainers review tests
# unsafe churn owner".
set -euo pipefail
out=$1
until=$2
purba=$3
here=$(cd "$(dirname "$0")" && pwd)

# owner in purba: level 1 needs a bot that proposes each version; level 2 a record that names
# when a person raises the crate. A script finds a cargo entry in dependabot.yml and the records
# that name the crate in backticks. It cannot read whether a record says when a person raises it.
bot=no
grep -qE 'package-ecosystem: *"?cargo' "${purba}/.github/dependabot.yml" 2>/dev/null && bot=yes
grep -rqlE 'renovate' "${purba}/.github" "${purba}/.config" 2>/dev/null && bot=yes

OWNERS='[(.users | map(select(.kind=="team")) | length),
  (.users | map(select(.kind=="user")) | length)] | @tsv'
# shellcheck disable=SC2016 # a jq program, which the shell must not expand
WARN='select(.reason=="compiler-message" and (.package_id | endswith($id))) | .message
  | [(.code.code // ""), .message, (.spans[0] | "\(.file_name):\(.line_start):\(.column_start)")]
  | @tsv'

while IFS=$'\t' read -r crate version parts; do
  probe=${out}/probe/${crate}
  # blast area
  if [[ -s "${probe}/tree.txt" ]]; then
    brings=$(wc -l <"${probe}/set.txt")
    if [[ "${brings}" -gt 30 ]]; then
      blast=3
    elif [[ "${parts}" -ge 2 ]] || [[ "${brings}" -gt 10 ]]; then
      blast=2
    else
      blast=1
    fi
  else
    blast=-
  fi

  # upkeep and API churn, from the crates.io versions published before UNTIL
  levels=$(jq -r --arg until "${until}" --arg v "${version}" -f "${here}/versions.jq" \
    "${out}/cratesio/${crate}.json")
  read -r upkeep churn <<<"${levels}"

  # maintainers: a team owner gives 1; otherwise the count of user owners
  owners=$(jq -r "${OWNERS}" "${out}/cratesio/${crate}.owners.json")
  read -r teams users <<<"${owners}"
  if [[ "${teams}" -ge 1 ]] || [[ "${users}" -ge 3 ]]; then
    maintainers=1
  elif [[ "${users}" -eq 2 ]]; then
    maintainers=2
  else
    maintainers=3
  fi

  # upstream review and tests, from the repository the crates.io record names
  repo=$(awk -F'\t' -v c="${crate}" '$1==c {print $2}' "${out}/github/repos.tsv")
  github=${out}/github/${repo/\//_}
  if [[ "${repo}" != "-" ]] && [[ -s "${github}/review.txt" ]]; then
    read -r reviewed _ <"${github}/review.txt"
    if [[ "${reviewed}" -ge 27 ]]; then
      review=1
    elif [[ "${reviewed}" -ge 15 ]]; then
      review=2
    else
      review=3
    fi
    observed=$(cat "${github}/tests.txt")
  else
    review=-
    observed=none
  fi
  source=$(cat "${probe}/srcdir.txt" 2>/dev/null || true)
  marked=0
  if [[ -n "${source}" ]]; then
    marked=$(grep -rE --include='*.rs' '#\[test\]' "${source}" | wc -l) || true
  fi
  case ${observed} in
    observed*) tests=1 ;;
    *)
      if [[ "${repo}" = "-" ]]; then
        tests=-
      elif [[ "${marked}" -gt 0 ]]; then
        tests=2
      else
        tests=3
      fi
      ;;
  esac

  # unsafe, from the forced lints on what the probe compiled
  check=${probe}/check.json
  unsafe=-
  if [[ -s "${check}" ]] &&
    jq -e 'select(.reason=="build-finished") | .success' "${check}" >/dev/null; then
    jq -r --arg id "#${crate}@${version}" "${WARN}" "${check}" | sort -u >"${probe}/warnings.tsv"
    blocks=$(awk -F'\t' '$1=="unsafe_code" && $2 ~ /`unsafe` block/' "${probe}/warnings.tsv" |
      wc -l)
    bare=$(awk -F'\t' '$1=="clippy::undocumented_unsafe_blocks" && $2 ~ /^unsafe block/' \
      "${probe}/warnings.tsv" | wc -l)
    if [[ "${blocks}" -eq 0 ]]; then
      unsafe=1
    elif [[ "${bare}" -eq 0 ]]; then
      unsafe=2
    else
      unsafe=3
    fi
  fi

  # owner in purba
  owner=3
  if [[ "${bot}" = yes ]] || grep -rqF -- "\`${crate}\`" "${purba}/docs/decisions" 2>/dev/null; then
    owner=-
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "${crate}" "${version}" "${blast}" \
    "${upkeep}" "${maintainers}" "${review}" "${tests}" "${unsafe}" "${churn}" "${owner}"
done <"${out}/crates.tsv"
