#!/usr/bin/env bash
# levels.sh OUT UNTIL PURBA_TREE
# Maps what fetch-cratesio.sh, run-probes.sh and run-github.sh wrote to a level from 1 to 3 for
# blast area and the seven risks, by the scales of
# docs/decisions/a-crate-joins-when-its-relief-exceeds-its-pain.md, read as
# research/future-crates.md reads them ("The primary reading of each input").
# "-" means the script gives no level.
# UNTIL cuts the crates.io versions. The windows are 3 months, 12 months and 2 years before its
# date. Prints "crate version blast upkeep maintainers review tests unsafe churn owner", and
# writes the numbers behind each level to OUT/evidence.md.
set -euo pipefail
out=$1
until=$2
purba=$3
here=$(cd "$(dirname "$0")" && pwd)
day=${until:0:10}
m3=$(date -u -d "${day} -3 months" +%F)
m12=$(date -u -d "${day} -12 months" +%F)
y2=$(date -u -d "${day} -24 months" +%F)

# owner in purba: level 1 needs a bot that proposes each version; level 2 a record that names
# when a person raises the crate. A script finds a cargo entry in dependabot.yml and the records
# that name the crate in backticks. It cannot read whether a record says when a person raises it.
bot=no
grep -qE 'package-ecosystem: *"?cargo' "${purba}/.github/dependabot.yml" && bot=yes
grep -rqlE 'renovate' "${purba}/.github" "${purba}/.config" 2>/dev/null && bot=yes

# the parts of a crate: each part of each need that lists the crate as a candidate
# shellcheck disable=SC2016 # an awk program, which the shell must not expand
PARTS='!/^#/ {
  n = split($3, a, " ")
  for (i = 1; i <= n; i++) {
    if (a[i] == c) { m = split($2, p, ","); for (j = 1; j <= m; j++) s[p[j]] = 1 }
  }
}
END { k = 0; for (x in s) k++; print k }'
OWNERS='[(.users | map(select(.kind=="team")) | length),
  (.users | map(select(.kind=="user")) | length),
  (.users | map(.login) | join(","))] | @tsv'
# shellcheck disable=SC2016 # a jq program, which the shell must not expand
WARN='select(.reason=="compiler-message" and (.package_id | endswith($id))) | .message
  | [(.code.code // ""), .message, (.spans[0] | "\(.file_name):\(.line_start):\(.column_start)")]
  | @tsv'

ev=${out}/evidence.md
{
  echo '| crate | blast | upkeep | maintainers | review | tests | unsafe | churn | owner |'
  echo '|---|---|---|---|---|---|---|---|---|'
} >"${ev}"
grep -v '^#' "${here}/crates.tsv" >"${out}/crates.list"
while IFS=$'\t' read -r c v; do
  # blast area
  parts=$(awk -F'\t' -v c="${c}" "${PARTS}" "${here}/needs.tsv")
  if [[ -s "${out}/probe/${c}/tree.txt" ]]; then
    brings=$(wc -l <"${out}/probe/${c}/set.txt")
    if [[ "${brings}" -gt 30 ]]; then
      bl=3
    elif [[ "${parts}" -ge 2 ]] || [[ "${brings}" -gt 10 ]]; then
      bl=2
    else
      bl=1
    fi
  else
    brings=-
    bl=-
  fi

  # upkeep and API churn, from the crates.io versions published before UNTIL
  vline=$(jq -r --arg until "${until}" --arg v "${v}" --arg m3 "${m3}" --arg m12 "${m12}" \
    --arg y2 "${y2}" -f "${here}/versions.jq" "${out}/cratesio/${c}.json")
  read -r up ch upev chev <<<"${vline}"

  # maintainers: a team owner gives 1; otherwise the count of user owners
  oline=$(jq -r "${OWNERS}" "${out}/cratesio/${c}.owners.json")
  read -r teams users logins <<<"${oline}"
  if [[ "${teams}" -ge 1 ]] || [[ "${users}" -ge 3 ]]; then
    mt=1
  elif [[ "${users}" -eq 2 ]]; then
    mt=2
  else
    mt=3
  fi

  # upstream review and tests, from the repository the crates.io record names
  repo=$(awk -F'\t' -v c="${c}" '$1==c {print $2}' "${out}/github/repos.tsv")
  G=${out}/github/${repo/\//_}
  if [[ "${repo}" != "-" ]] && [[ -s "${G}/review.txt" ]]; then
    read -r rvd rvn rvfrom rvto <"${G}/review.txt"
    if [[ "${rvd}" -ge 27 ]]; then
      rv=1
    elif [[ "${rvd}" -ge 15 ]]; then
      rv=2
    else
      rv=3
    fi
    rvev="${rvd}/${rvn} ${rvfrom}..${rvto}"
    tobs=$(cat "${G}/tests.txt")
  else
    rv=-
    rvev="no GitHub repository (${repo})"
    tobs=none
  fi
  src=$(cat "${out}/probe/${c}/srcdir.txt" 2>/dev/null || true)
  ntest=0
  if [[ -n "${src}" ]]; then
    grep -rE --include='*.rs' '#\[test\]' "${src}" >"${out}/probe/${c}/tests.txt" || true
    ntest=$(wc -l <"${out}/probe/${c}/tests.txt")
  fi
  case ${tobs} in
    observed*) ts=1 ;;
    *)
      if [[ "${repo}" = "-" ]]; then
        ts=-
      elif [[ "${ntest}" -gt 0 ]]; then
        ts=2
      else
        ts=3
      fi
      ;;
  esac

  # unsafe, from the forced lints on what the probe compiled
  chk=${out}/probe/${c}/check.json
  W=${out}/probe/${c}/warnings.tsv
  built=no
  if [[ -s "${chk}" ]]; then
    jq -e 'select(.reason=="build-finished") | .success' "${chk}" >/dev/null && built=yes
  fi
  if [[ "${built}" = yes ]]; then
    jq -r --arg id "#${c}@${v}" "${WARN}" "${chk}" | sort -u >"${W}"
    nb=$(awk -F'\t' '$1=="unsafe_code" && $2 ~ /`unsafe` block/' "${W}" | wc -l)
    nu=$(awk -F'\t' '$1=="clippy::undocumented_unsafe_blocks" && $2 ~ /^unsafe block/' "${W}" |
      wc -l)
    if [[ "${nb}" -eq 0 ]]; then
      us=1
    elif [[ "${nu}" -eq 0 ]]; then
      us=2
    else
      us=3
    fi
  else
    nb=-
    nu=-
    us=-
  fi

  # owner in purba
  grep -rlF -- "\`${c}\`" "${purba}/docs/decisions" >"${out}/owner-records.txt" 2>/dev/null || true
  recs=$(xargs -r -n1 basename <"${out}/owner-records.txt" | paste -sd, -)
  if [[ "${bot}" = yes ]]; then
    ow=-
    owev="a cargo bot entry exists; whether it proposes this crate is a reading"
  elif [[ -n "${recs}" ]]; then
    ow=-
    owev="no cargo bot; records name it: ${recs}"
  else
    ow=3
    owev="no cargo bot; no record names it"
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${c}" "${v}" "${bl}" "${up}" "${mt}" "${rv}" "${ts}" "${us}" "${ch}" "${ow}"
  # shellcheck disable=SC2016 # the backticks are Markdown, not a command
  printf '| `%s` %s | %s parts, brings %s | %s | teams=%s users=%s (%s) | %s | %s; `#[test]`: %s' \
    "${c}" "${v}" "${parts}" "${brings}" "${upev}" "${teams}" "${users}" "${logins}" "${rvev}" \
    "${tobs}" "${ntest}" >>"${ev}"
  printf ' | %s blocks, %s without SAFETY | %s | %s |\n' \
    "${nb}" "${nu}" "${chev}" "${owev}" >>"${ev}"
done <"${out}/crates.list"
