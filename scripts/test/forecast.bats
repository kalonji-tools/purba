# scripts/forecast.sh against recorded crates.io answers, with fakes for the build and GitHub.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../forecast.sh"
  make_repo
  answers="${BATS_TEST_TMPDIR}/answers"
  mkdir -p .config "${answers}"
  : >.config/needs.toml
  : >.config/shortlists.toml

  # crates.io answers a keyword with the file keyword-<word>.json, and with no
  # crate when there is none. It answers a crate with its newest version, 1.2.0,
  # the first of its major version, 1.0.0, and an owner that is a team.
  fake curl <<'FAKE'
url=${*: -1}
answers="${BATS_TEST_TMPDIR}/answers"
case ${url} in
  *'/api/v1/crates?keyword='*)
    word=${url#*keyword=}
    word=${word%%&*}
    if [[ -e "${answers}/keyword-${word}.json" ]]; then
      cat "${answers}/keyword-${word}.json"
    else
      printf '{"crates": []}\n'
    fi
    ;;
  */owners) printf '{"users": [{"login": "github:o:t", "kind": "team"}]}\n' ;;
  *)
    name=${url##*/}
    printf '{"crate": {"name": "%s", "repository": "https://github.com/o/%s"},\n' \
      "${name}" "${name}"
    printf ' "versions": [{"num": "1.2.0", "yanked": false,'
    printf ' "created_at": "2026-11-01T08:00:00Z"},\n'
    printf ' {"num": "1.0.0", "yanked": false, "created_at": "2024-01-01T08:00:00Z"}]}\n'
    ;;
esac
FAKE

  # The build finds nothing the crate brings beside the probe, and no unsafe block.
  fake cargo <<'FAKE'
case $1 in
  generate-lockfile)
    manifest=${*: -1}
    grep -F '= "=' "${manifest}" >>"${BATS_TEST_TMPDIR}/probed"
    ;;
  tree) printf 'probe_crate v0.0.0\n' ;;
  metadata) printf '{"packages": []}\n' ;;
  check) printf '{"reason": "build-finished", "success": true}\n' ;;
esac
exit 0
FAKE
  fake cargo-deny <<'FAKE'
printf '{"type": "summary", "fields": {}}\n' >&2
FAKE
  fake rustup <<'FAKE'
command -v true
FAKE
  # GitHub holds no commit and no workflow.
  fake gh <<'FAKE'
case "$*" in
  'auth token') printf 'token\n' ;;
  *review.graphql*)
    printf '{"data": {"repository": {"defaultBranchRef":\n'
    printf ' {"target": {"history": {"nodes": []}}}}}}\n'
    ;;
  *workflows.graphql*) printf '{"data": {"repository": {"object": null}}}\n' ;;
  *) exit 1 ;;
esac
FAKE
  fake sleep <<<''
}

# A line of .config/needs.toml for the need <key> of <part>, with each extra
# field as it is given, such as declined = [].
need() {
  local key=$1 part=$2 words=$3 extra=${4:+, $4}
  printf '%s = { part = "%s", need = "do %s", words = %s, stated = "%s"%s }\n' \
    "${key}" "${part}" "${key}" "${words}" https://example.invalid "${extra}" >>.config/needs.toml
}

# crates.io finds each <crate> for <word>.
finds() {
  local word=$1 crates
  shift
  crates=$(printf '{"name": "%s"},' "$@")
  printf '{"crates": [%s]}\n' "${crates%,}" >"${answers}/keyword-${word}.json"
}

# An entry of .config/shortlists.toml for <crate> under <need>, at version 1.2.0,
# measured on 2026-11-14, with the levels the fakes give, except each
# <level>=<value>.
shortlisted() {
  local need=$1 crate=$2 line
  shift 2
  local -A level=(['blast-area']=1 ['upkeep']=1 ['maintainers']=1 ['upstream-review']=3
    ['tests']=3 ['unsafe']=1 ['api-churn']=1 ['owner-in-purba']=3 ['needs']=2)
  for line in "$@"; do
    level[${line%%=*}]=${line#*=}
  done
  {
    printf '[%s.%s]\nversion = "1.2.0"\nmeasured = "2026-11-14"\n' "${need}" "${crate}"
    for line in blast-area upkeep maintainers upstream-review tests unsafe api-churn \
      owner-in-purba; do
      printf '%s = %s\n' "${line}" "${level[${line}]}"
    done
    printf 'unmeasured = []\nneeds = %s\n\n' "${level[needs]}"
  } >>.config/shortlists.toml
}

@test "a new line gets an issue body that names each crate its words find" {
  need read-flags command '["cli", "args"]'
  finds cli clap lexopt
  finds args clap pico-args

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"==> read-flags <=="* ]]
  [[ "${output}" == *"do read-flags"* ]]
  [[ "${output}" == *"| \`clap\` | 1.2.0 |"* ]]
  [[ "${output}" == *"| \`lexopt\` | 1.2.0 |"* ]]
  [[ "${output}" == *"| \`pico-args\` | 1.2.0 |"* ]]
  rows=$(grep -c "^| \`clap\`" <<<"${output}")
  [[ "${rows}" -eq 1 ]]
}

@test "the query asks for the top 20 by recent downloads, and names purba and no person" {
  need read-flags command '["cli"]'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  asked=$(calls curl)
  query="https://crates.io/api/v1/crates?keyword=cli&sort=recent-downloads&per_page=20"
  [[ "${asked}" == *"${query}"* ]]
  [[ "${asked}" == *"purba (https://github.com/kalonji-tools/purba)"* ]]
}

@test "a shortlisted line, a line with an issue and a declined line get no forecast" {
  need read-flags command '["cli"]'
  shortlisted read-flags clap
  need find-files selection '["walk"]' 'issue = "https://example.invalid/issues/1"'
  need write-junit reporting '["junit"]' 'declined = ["junit-report"]'
  finds cli clap
  finds walk walkdir
  finds junit quick-junit

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
  asked=$(calls curl)
  [[ -z "${asked}" ]]
}

@test "a line whose words find no crate needs declined = [], and the forecast does not write it" {
  need write-junit reporting '["junit"]'
  cp .config/needs.toml "${BATS_TEST_TMPDIR}/needs.before"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"write-junit: no crate; write declined = []"* ]]
  [[ "${output}" == *"write-junit: the word junit finds no crate"* ]]
  [[ "${output}" != *"==> write-junit <=="* ]]
  cmp .config/needs.toml "${BATS_TEST_TMPDIR}/needs.before"
}

@test "--part reads the declined lines of its part, and drops each declined crate" {
  need write-junit reporting '["junit"]' 'declined = ["junit-report"]'
  finds junit junit-report quick-junit

  run "${script}" --part reporting

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"==> write-junit <=="* ]]
  [[ "${output}" == *"| \`quick-junit\` |"* ]]
  [[ "${output}" != *"\`junit-report\`"* ]]
}

@test "--part reads no line of another part" {
  need write-junit reporting '["junit"]'
  need read-flags command '["cli"]'
  finds junit quick-junit
  finds cli clap

  run "${script}" --part reporting

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"==> write-junit <=="* ]]
  [[ "${output}" != *"read-flags"* ]]
  asked=$(calls curl)
  [[ "${asked}" != *"keyword=cli"* ]]
}

@test "a declined line whose words find only declined crates keeps its declined list" {
  need write-junit reporting '["junit"]' 'declined = ["junit-report"]'
  finds junit junit-report

  run "${script}" --part reporting

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *'write-junit: no crate beyond declined = ["junit-report"]'* ]]
  [[ "${output}" != *"==> write-junit <=="* ]]
}

@test "--shortlisted prints only the entries whose levels moved, each as old→new" {
  need read-flags command '["cli"]'
  shortlisted read-flags clap
  shortlisted read-flags lexopt blast-area=3 maintainers=3 tests=1 needs=3

  run "${script}" --shortlisted

  [[ "${status}" -eq 0 ]]
  moved="blast-area 3→1, maintainers 3→1, tests 1→3, needs 3→2"
  [[ "${output}" == *"read-flags.lexopt 1.2.0: ${moved}"* ]]
  [[ "${output}" != *"read-flags.clap"* ]]
  asked=$(calls curl)
  [[ "${asked}" != *"keyword="* ]]
}

@test "--shortlisted measures the version of each entry, with the cut at its measured date" {
  need read-flags command '["cli"]'
  shortlisted read-flags clap

  run "${script}" --shortlisted

  [[ "${status}" -eq 0 ]]
  grep -Fx 'clap = "=1.2.0"' "${BATS_TEST_TMPDIR}/probed"
  asked=$(calls gh)
  [[ "${asked}" == *"until=2026-11-14T00:00:00Z"* ]]
}

@test "the forecast leaves no scratch directory behind" {
  need read-flags command '["cli"]'
  finds cli clap

  run "${script}"

  [[ "${status}" -eq 0 ]]
  left=$(find "${TMPDIR}" -mindepth 1)
  [[ -z "${left}" ]]
}

@test "an unknown argument cannot run" {
  run "${script}" --everything

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"--everything"* ]]
}

@test "the licence cell counts the crates that fail, by finding" {
  need read-flags command '["cli"]'
  finds cli clap
  fake cargo-deny <<'FAKE'
# finding <code> <crate>: an error of cargo-deny on <crate>.
finding() {
  local crate="{\"name\": \"$2\", \"version\": \"1.0.0\"}"
  printf '{"type": "diagnostic", "fields": {"severity": "error", "code": "%s",' "$1" >&2
  printf ' "graphs": [{"Krate": %s}]}}\n' "${crate}" >&2
}
if [[ "${*: -1}" == licenses ]]; then
  finding rejected alpha
  finding rejected beta
  finding unlicensed gamma
fi
printf '{"type": "summary", "fields": {}}\n' >&2
FAKE

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"| \`clap\` | 1.2.0 | pass | 2 rejected, 1 unlicensed | pass |"* ]]
}
