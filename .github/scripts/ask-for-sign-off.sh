# shellcheck shell=bash
# How a workflow asks a person to sign a change a machine wrote.
#
#   what you owe:  CONTRIBUTING.md
#
#   sign_off_start <script> <workflow> <branch> <issue>
#       checks the run and a clean tree, and rewinds to that tree on exit
#   stand_down_while_open <what>
#       exits 0 while a pull request is open on <branch>
#   ask_for_sign_off <subject> <what> <from> <to> <closing> <file>...
#       refuses a change beyond <file>..., commits them as the bot, pushes
#       <branch>, and opens the pull request
#
#   GH_REPO    the repository `gh` acts on
#   GH_TOKEN   a token that may open a pull request
#
# A failure exits 2, and a change beyond <file>... exits 1.

# shellcheck source=scripts/report.sh
. "$(dirname "${BASH_SOURCE[0]}")/../../scripts/report.sh"

as_bot=(
  -c "user.name=github-actions[bot]"
  -c "user.email=41898282+github-actions[bot]@users.noreply.github.com"
)

sign_off_start() {
  local script=$1 dirty
  workflow=$2
  if [[ $# -ne 4 ]]; then
    cannot "usage: ${script} <branch> <issue>"
  fi

  if [[ -z "${GH_REPO:-}" ]]; then
    cannot "GH_REPO is set by the workflow env, and it is empty here."
  fi

  branch=$3
  issue=$4

  # The subject rule requires a reference, so a subject cannot be built without
  # one. docs/decisions/a-commit-outlives-its-review.md
  case "${issue}" in
    '' | 0 | *[!0-9]*)
      cannot "${script} needs an issue number, and was given '${issue}'."
      ;;
    *) ;;
  esac

  if ! git diff --quiet HEAD --; then
    dirty=$(git --no-pager status --short)
    cannot "${script} needs a clean tree, because it rewinds to one." "${dirty}"
  fi

  checked_out=$(git rev-parse HEAD)
  trap 'git reset --quiet --hard "${checked_out}"' EXIT
}

stand_down_while_open() {
  local open
  # ⚠️ Never replace this with a force-push. A pull request that broke keeps
  # the head that refused it, and the logs hanging off that head.
  if ! open=$(gh pr list --repo "${GH_REPO}" --head "${branch}" --state open --json number --jq \
    '.[].number'); then
    cannot "the open pull requests on ${branch} could not be read."
  fi
  if [[ -n "${open}" ]]; then
    echo "pull request #${open} is already proposing $1 on ${branch}, so this run stands down"
    echo "nothing is proposed until a person signs that one or closes it"
    exit 0
  fi
}

# Each argument, as a sentence lists them: `a`, `b` and `c`.
listed() {
  local all=$1
  while [[ $# -gt 2 ]]; do
    shift
    all+=", $1"
  done
  [[ $# -eq 1 ]] || all+=" and $2"
  printf '%s' "${all}"
}

ask_for_sign_off() {
  local subject=$1 what=$2 from=$3 to=$4 closing=$5
  local files=("${@:6}") others=() quoted=() file named shown changed body
  for file in "${files[@]}"; do
    others+=(":!${file}")
    quoted+=("\`${file}\`")
  done

  # ⚠️ Refuse a tracked file this changed and did not ask for. The commit below
  # names the files it takes, so nothing else reaches the proposal.
  if ! git diff --quiet -- . "${others[@]}"; then
    changed=$(git --no-pager diff --stat)
    named=$(listed "${files[@]}")
    refuse "the run changed files beyond ${named}, so it is not proposed" "${changed}"
    finish
  fi

  # ⚠️ No `-s`. CONTRIBUTING.md: a machine never writes that trailer.
  git add -- "${files[@]}"
  if ! git "${as_bot[@]}" commit --quiet -m "${subject}" -- "${files[@]}"; then
    cannot "the proposal could not be committed"
  fi

  # The branch outlives a closed pull request, so this replaces it.
  if ! git push --force origin "HEAD:refs/heads/${branch}"; then
    cannot "the proposal could not be pushed to ${branch}"
  fi

  shown=$(listed "${quoted[@]}")
  body=$(
    cat <<BODY
A machine wrote this. It proposes ${what} and certifies nothing.

| | |
|---|---|
| from | \`${from}\` |
| to | \`${to}\` |
| files | ${shown} |

⚠️ **\`Origin\` is red on this branch, and that is correct.** A machine never writes a \
\`Signed-off-by:\` trailer, so a person adds it to this commit:

\`\`\`
git fetch origin ${branch}
git switch --detach FETCH_HEAD
mise run sign-off
git push --force-with-lease origin HEAD:${branch}
\`\`\`

${closing}

Opened by \`${workflow}\`, which \
[#${issue}](https://github.com/${GH_REPO}/issues/${issue}) owns.
BODY
  )

  if ! gh pr create --repo "${GH_REPO}" --base main --head "${branch}" \
    --title "${subject}" \
    --body "${body}"; then
    cannot "${branch} is pushed, and its pull request could not be opened"
  fi
}
