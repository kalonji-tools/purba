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
#   gh_refuses <call>              that fake fails <call> with exit 1, as gh does
#   check_runs [<run>...]          `gh_answers check-runs`, one run for each
#                                  `<name>=<conclusion>[@<day>]`. An empty
#                                  conclusion is a run still going
: "${BATS_TEST_TMPDIR:?set by bats}"

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
  write="" path=""
  for argument in "$@"; do
    case "${argument}" in
      -f | --method) write=post- ;;
      repos/*) path=${argument%%\?*} ;;
      *) ;;
    esac
  done
  [[ "${path##*/}" == *[!0-9]* ]] || path=${path%/*}
  call=${write}${path##*/}
fi
answers="${BATS_TEST_TMPDIR}/gh"
if [[ -e "${answers}/${call}.refused" ]]; then
  echo "HTTP 502: ${call}" >&2
  exit 1
fi
case "${call}" in
  post-* | issue-create | issue-comment) echo "https://example.invalid/${call}" ;;
  *)
    if [[ ! -e "${answers}/${call}.json" ]]; then
      echo "the fake does not answer ${call}: $*" >&2
      exit 9
    fi
    cat "${answers}/${call}.json"
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
  local run name conclusion day one all=""
  for run in "$@"; do
    name=${run%%=*}
    conclusion=${run#*=}
    day=2026-01-01
    if [[ "${conclusion}" == *@* ]]; then
      day=${conclusion#*@}
      conclusion=${conclusion%@*}
    fi
    if [[ -n "${conclusion}" ]]; then
      printf -v one '{"name":"%s","conclusion":"%s","completed_at":"%sT00:00:00Z"}' \
        "${name}" "${conclusion}" "${day}"
    else
      printf -v one '{"name":"%s","conclusion":null,"completed_at":null}' "${name}"
    fi
    all+="${all:+,}${one}"
  done
  gh_answers check-runs "{\"check_runs\":[${all}]}"
}
