# shellcheck shell=bash
# The repository a test builds, and the fake commands a script finds on PATH.
#
#   the decision:  docs/decisions/purba-writes-its-scripts-in-bash-until-purba-can-test-them.md
#   the task:      mise run test:shell
#
#   isolate                        nothing from the machine, and nothing from the
#                                  repository this suite is run in
#   make_repo                      a repository whose `main` holds one commit and
#                                  one file, `chorestart`
#   commit <subject> [<line>...]   a commit whose message ends with each line
#   fake <command>                 a fake first on PATH, its body on standard input
#   calls <command>                each call that fake received, its arguments
#                                  joined by spaces, and nothing if it received none
#   git_fails_on <word>            a fake git that exits 128 on a call with the
#                                  argument <word>, and runs every other
#   fake_gh                        a fake `gh` that answers each read <call> with
#                                  the JSON of `gh_answers <call>`. <call> is
#                                  `issue-list`, or the last segment of an API path
#                                  that is not a number. A write is answered with a
#                                  URL, and is `post-<call>`, `issue-create` or
#                                  `issue-comment`
#   gh_answers <call> <json>       what that fake answers <call> with
#   gh_refuses <call>              that fake fails <call> with exit 1, as gh does.
#                                  <call> may be `post-check-runs-<name>`, a post
#                                  of the check run <name> alone
#   check_runs [<run>...]          `gh_answers check-runs`, one run for each
#                                  `<name>=<conclusion>[@<day>][~<source>]`. An
#                                  empty conclusion is a run still going. It also
#                                  answers `runs` with the workflow run behind
#                                  each check run that one made, and `jobs` with
#                                  the job behind it. <source> is:
#                                    own     the job of the gate's own workflow,
#                                            on this pull request (the default)
#                                    suite   a post into that same run, listed
#                                            as a job that ran on no runner
#                                    api     a post, which no workflow run made
#                                    push    the gate's workflow, on a push
#                                    branch  the gate's workflow, on a pull
#                                            request from another branch
#                                    file    another workflow file, on this pull
#                                            request
#                                    sign    a post by `purba-sign`
#                                  Each workflow run it makes holds the artifact
#                                  `gate-token-1`, a token that `gate` signed. Its
#                                  claims name that run, its job, `PR` and
#                                  `BASE_REF` (7 and `main` when unset). A fake
#                                  `curl` answers with the key set
#   gate_token <run> <filter> [<key> [<alg> [<kid>]]]
#                                  signs that token of <run> again, with its claims
#                                  passed through the jq <filter>. <key> is `gate`,
#                                  `other`, which no key set holds, or `short`, of
#                                  1024 bits. An <alg> of `none` signs nothing.
#                                  <kid> is the key that the header names.
#                                  Without it, the header names <key>
#
# An answer of `gh_answers <call>-<id>` goes to a call whose path holds the
# number <id>. The answer of `gh_answers <call>` goes to every other call.
: "${BATS_TEST_TMPDIR:?set by bats}"
: "${BATS_FILE_TMPDIR:?set by bats}"

# ⚠️ git hands a hook, and a command under `rebase --exec`, the repository it
# runs in. A test that keeps those variables commits into that repository.
#
# A signing key, a hook or an ignore file on the machine changes what git does
# in a test as well, so git reads nothing from outside it and cannot climb out
# of it. A script's own scratch directory lands inside it too.
#
# `report` writes an annotation on a runner and plain text everywhere else, so a
# test that leaves the variable alone reads a different message in CI. A tool
# can print a different line when FORCE_COLOR is set, and no workflow sets it.
isolate() {
  local names
  local -a handed
  names=$(git rev-parse --local-env-vars)
  mapfile -t handed <<<"${names}"
  unset "${handed[@]}"

  export HOME="${BATS_TEST_TMPDIR}/home" TMPDIR="${BATS_TEST_TMPDIR}/tmp"
  export XDG_CONFIG_HOME="${HOME}/.config"
  mkdir -p "${HOME}" "${TMPDIR}"
  export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
  export GIT_CEILING_DIRECTORIES="${BATS_TEST_TMPDIR}"
  export GIT_AUTHOR_NAME=writer GIT_AUTHOR_EMAIL=writer@example.invalid
  export GIT_COMMITTER_NAME=writer GIT_COMMITTER_EMAIL=writer@example.invalid
  unset GITHUB_ACTIONS PURBA_REPORT FORCE_COLOR
}

make_repo() {
  isolate
  git init --quiet --initial-branch=main "${BATS_TEST_TMPDIR}/repo"
  cd "${BATS_TEST_TMPDIR}/repo" || return 1
  commit "chore: start"
}

# Each commit appends its subject to a file named after it, so two commits never
# collide and none is empty.
commit() {
  local subject=$1 message=$1 file
  shift
  [[ $# -eq 0 ]] || message+=$'\n\n'$(printf '%s\n' "$@")
  file=${subject//[^a-z]/}
  printf '%s\n' "${subject}" >>"${file}"
  git add -- "${file}"
  git commit --quiet --message "${message}"
}

fake() {
  mkdir -p "${BATS_TEST_TMPDIR}/bin"
  {
    printf '#!/usr/bin/env bash\n'
    # The fake expands these when a script runs it, and this function must not.
    # shellcheck disable=SC2016
    printf 'printf "%%s\\n" "$*" >>"${BATS_TEST_TMPDIR}/calls.%s"\n' "$1"
    cat
  } >"${BATS_TEST_TMPDIR}/bin/$1"
  chmod +x "${BATS_TEST_TMPDIR}/bin/$1"
  PATH="${BATS_TEST_TMPDIR}/bin:${PATH}"
}

calls() {
  [[ ! -e "${BATS_TEST_TMPDIR}/calls.$1" ]] || cat "${BATS_TEST_TMPDIR}/calls.$1"
}

git_fails_on() {
  local git
  git=$(command -v git)
  fake git <<FAKE
for argument in "\$@"; do
  [[ "\${argument}" != "$1" ]] || { echo "fatal: git cannot $1" >&2; exit 128; }
done
exec "${git}" "\$@"
FAKE
}

fake_gh() {
  fake gh <<'FAKE'
call="$1-$2"
if [[ "$1" == api ]]; then
  write="" path="" named="" id=""
  for argument in "$@"; do
    case "${argument}" in
      -f | --method) write=post- ;;
      repos/*) path=${argument%%\?*} ;;
      name=*) named=${argument#name=} ;;
      *) ;;
    esac
  done
  [[ ! "${path}" =~ /([0-9]+)(/[^/]*[^/0-9][^/]*)?$ ]] || id=${BASH_REMATCH[1]}
  [[ "${path##*/}" == *[!0-9]* ]] || path=${path%/*}
  call=${write}${path##*/}
  [[ "${path}" != */commits/*/pulls ]] || call=${write}commit-pulls
fi
answers="${BATS_TEST_TMPDIR}/gh"
if [[ -e "${answers}/${call}.refused" || -e "${answers}/${call}-${named:-}.refused" ]]; then
  echo "HTTP 502: ${call}" >&2
  exit 1
fi
case "${call}" in
  post-* | issue-create | issue-comment) echo "https://example.invalid/${call}" ;;
  *)
    answer="${answers}/${call}.json"
    keyed="${answers}/${call}-${id:-}.json"
    [[ -z "${id:-}" || ! -e "${keyed}" ]] || answer=${keyed}
    if [[ ! -e "${answer}" ]]; then
      echo "the fake does not answer ${call}: $*" >&2
      exit 9
    fi
    cat "${answer}"
    ;;
esac
FAKE
}

gh_answers() {
  mkdir -p "${BATS_TEST_TMPDIR}/gh"
  printf '%s\n' "$2" >"${BATS_TEST_TMPDIR}/gh/$1.json"
}

gh_refuses() {
  mkdir -p "${BATS_TEST_TMPDIR}/gh"
  touch "${BATS_TEST_TMPDIR}/gh/$1.refused"
}

check_runs() {
  local run name conclusion day source file event branch app runner suite=0 one made \
    job all="" workflows="" jobs=""
  for run in "$@"; do
    name=${run%%=*}
    conclusion=${run#*=}
    source=own
    if [[ "${conclusion}" == *~* ]]; then
      source=${conclusion#*~}
      conclusion=${conclusion%~*}
    fi
    day=2026-01-01
    if [[ "${conclusion}" == *@* ]]; then
      day=${conclusion#*@}
      conclusion=${conclusion%@*}
    fi
    suite=$((suite + 1))
    file=${name%%-*}
    file=${file,,}.yml
    app=github-actions
    made=""
    runner=1
    case "${source}" in
      own) made="${file} pull_request ${HEAD_REF:-}" ;;
      suite)
        made="${file} pull_request ${HEAD_REF:-}"
        runner=null
        ;;
      push) made="${file} push other" ;;
      branch) made="${file} pull_request other" ;;
      file) made="forge.yml pull_request ${HEAD_REF:-}" ;;
      api) ;;
      sign) app=purba-sign ;;
      *) return 1 ;;
    esac
    if [[ -n "${made}" ]]; then
      read -r file event branch <<<"${made}"
      printf -v one '{"id":%d,"check_suite_id":%d,"path":".github/workflows/%s",' \
        "$((100 + suite))" "${suite}" "${file}"
      printf -v one '%s"event":"%s","head_branch":"%s"}' "${one}" "${event}" "${branch}"
      workflows+="${workflows:+,}${one}"
    fi
    if [[ -n "${conclusion}" ]]; then
      printf -v one '"name":"%s","conclusion":"%s","completed_at":"%sT00:00:00Z"' \
        "${name}" "${conclusion}" "${day}"
    else
      printf -v one '"name":"%s","conclusion":null,"completed_at":null' "${name}"
    fi
    # A job's id is the id of its check run. Every run's read of its jobs gets
    # them all here, so a job names its run.
    if [[ -n "${made}" ]]; then
      printf -v job '{"id":%d,"run_id":%d,"run_attempt":1,%s,"runner_id":%s}' \
        "$((1000 + suite))" "$((100 + suite))" "${one}" "${runner}"
      jobs+="${jobs:+,}${job}"
      gate_token_for "$((100 + suite))" "$((1000 + suite))"
    fi
    printf -v one '{"id":%d,%s,"app":{"slug":"%s"},"check_suite":{"id":%d}}' \
      "$((1000 + suite))" "${one}" "${app}" "${suite}"
    all+="${all:+,}${one}"
  done
  gh_answers check-runs "{\"check_runs\":[${all}]}"
  gh_answers runs "{\"workflow_runs\":[${workflows}]}"
  gh_answers jobs "{\"jobs\":[${jobs}]}"
}

# This function makes a key set like GitHub's. A fake `curl` answers with it.
# The set holds `gate`, and `short`, a key of 1024 bits. The key `other` is
# in no set. This function makes the keys once for each file of tests, because
# a key of 2048 bits is slow to make.
gate_keys() {
  local dir="${BATS_FILE_TMPDIR}/gate-keys" name bits gate short
  # `bats --jobs` runs the tests of one file at once, and each may come here
  # first. `mkdir` is atomic, so one test makes the keys while the rest wait. That
  # test writes the key set last.
  if [[ ! -e "${dir}/jwks.json" ]]; then
    until mkdir "${dir}.lock" 2>/dev/null; do sleep 0.1; done
    if [[ ! -e "${dir}/jwks.json" ]]; then
      mkdir -p "${dir}"
      for name in gate:2048 other:2048 short:1024; do
        bits=${name#*:}
        name=${name%:*}
        openssl genpkey -algorithm RSA -pkeyopt "rsa_keygen_bits:${bits}" \
          -out "${dir}/${name}.pem" 2>/dev/null
      done
      gate=$(gate_modulus "${dir}/gate.pem")
      short=$(gate_modulus "${dir}/short.pem")
      jq -n --arg gate "${gate}" --arg short "${short}" '{keys: [
          {kid: "gate", kty: "RSA", alg: "RS256", use: "sig", e: "AQAB", n: $gate},
          {kid: "short", kty: "RSA", alg: "RS256", use: "sig", e: "AQAB", n: $short}]}' \
        >"${dir}/jwks.partial"
      mv "${dir}/jwks.partial" "${dir}/jwks.json"
    fi
    rmdir "${dir}.lock"
  fi
  fake curl <<FAKE
cat "${dir}/jwks.json"
FAKE
  # An artifact reaches the script as a zip. Here it is the token itself.
  fake unzip <<'FAKE'
cat "$2"
FAKE
}

# The modulus of a key, in base64url.
gate_modulus() {
  local hex escaped="" i
  hex=$(openssl rsa -in "$1" -noout -modulus)
  hex=${hex#Modulus=}
  for ((i = 0; i < ${#hex}; i += 2)); do escaped+="\\x${hex:i:2}"; done
  printf '%b' "${escaped}" >"$1.modulus"
  gate_b64url "$1.modulus"
}

# The base64url of the bytes in a file, with no padding.
gate_b64url() {
  local encoded
  encoded=$(base64 <"$1")
  encoded=${encoded//$'\n'/}
  encoded=${encoded//=/}
  encoded=${encoded//+/-}
  printf '%s' "${encoded//\//_}"
}

# gate_token_for <run> <job>
#
# Signs the token of the job <job> of the workflow run <run> by `gate`. It also
# makes the fake `gh` answer with the artifact `gate-token-1` of that run, whose
# id is <run> + 5000.
# `gate_token` changes the claims.
gate_token_for() {
  local claims
  gate_keys
  claims=$(jq -nc --arg repo "${GH_REPO:-}" --arg ref "refs/pull/${PR:-7}/merge" \
    --arg base "${BASE_REF:-main}" --arg run "$1" --arg job "$2" '{
      iss: "https://token.actions.githubusercontent.com", aud: "purba-gate",
      repository: $repo, event_name: "pull_request", ref: $ref, base_ref: $base,
      run_id: $run, run_attempt: "1", check_run_id: $job}')
  printf '%s\n' "${claims}" >"${BATS_TEST_TMPDIR}/claims-$1"
  gh_answers "artifacts-$1" \
    "{\"artifacts\":[{\"id\":$((5000 + $1)),\"name\":\"gate-token-1\",\"expired\":false}]}"
  gate_sign "$1" gate RS256
}

gate_token() {
  local claims
  claims=$(jq -c "$2" "${BATS_TEST_TMPDIR}/claims-$1")
  printf '%s\n' "${claims}" >"${BATS_TEST_TMPDIR}/claims-$1"
  gate_sign "$1" "${3:-gate}" "${4:-RS256}" "${5:-${3:-gate}}"
}

gate_sign() {
  local header payload signature="" key="${BATS_FILE_TMPDIR}/gate-keys/$2.pem"
  local dir="${BATS_TEST_TMPDIR}/jwt-$1"
  mkdir -p "${dir}"
  printf '{"alg":"%s","kid":"%s","typ":"JWT"}' "$3" "${4:-$2}" >"${dir}/header"
  tr -d '\n' <"${BATS_TEST_TMPDIR}/claims-$1" >"${dir}/payload"
  header=$(gate_b64url "${dir}/header")
  payload=$(gate_b64url "${dir}/payload")
  if [[ "$3" != none ]]; then
    printf '%s.%s' "${header}" "${payload}" >"${dir}/signed"
    openssl dgst -sha256 -sign "${key}" -out "${dir}/signature" "${dir}/signed"
    signature=$(gate_b64url "${dir}/signature")
  fi
  gh_answers "zip-$((5000 + $1))" "${header}.${payload}.${signature}"
}
