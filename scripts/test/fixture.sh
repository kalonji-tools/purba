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
#   fake_gh                        a fake `gh` that answers the check-runs API
#   check_runs [<run>...]          what that fake answers a read with, one run for
#                                  each `<name>=<conclusion>[@<day>]`. An empty
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

fake_gh() {
  fake gh <<'FAKE'
case "$*" in
  *"--method POST"*) echo "https://example.invalid/check-run" ;;
  *) cat "${BATS_TEST_TMPDIR}/check-runs" ;;
esac
FAKE
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
  printf '{"check_runs":[%s]}\n' "${all}" >"${BATS_TEST_TMPDIR}/check-runs"
}
