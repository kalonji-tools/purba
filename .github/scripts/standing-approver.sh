# shellcheck shell=bash
# Which approval on a pull request an Accepted-by trailer is written from.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#
#   standing_approver
#       reads the reviews of one pull request on standard input, and prints
#       `<login> <id>` of the approval that stands, or nothing when none stands.
#       It reads the permission of each approver from GitHub, and only admin
#       or write stands.
#
#   GH_REPO    the repository whose permissions it reads. A call without it
#              exits 2.
#
# A login or an id that no trailer may hold exits 1. Reviews the rule cannot
# read, or a permission it cannot read, exit 2.

# shellcheck source=scripts/report.sh
. "$(dirname "${BASH_SOURCE[0]}")/../../scripts/report.sh"

standing_approver() {
  local reviews standing login id answer permission
  [[ -n "${GH_REPO:-}" ]] || cannot "GH_REPO is set by the workflow env, and it is empty here."
  reviews=$(cat)
  if ! jq -e 'type == "array"' >/dev/null 2>&1 <<<"${reviews}"; then
    cannot "the reviews of this pull request are not a JSON array, so no approval was read."
  fi
  if ! standing=$(jq -r '
    to_entries | map(.value + {order: .key})
    | map(select(.state == "APPROVED" or .state == "CHANGES_REQUESTED" or .state == "DISMISSED"))
    | group_by(.user.id) | map(max_by(.order)) | map(select(.state == "APPROVED"))
    | map(select(.user.login != null)) | sort_by(.order) | reverse | .[]
    | "\(.user.login)\t\(.user.id // "")"' <<<"${reviews}"); then
    cannot "the reviews of this pull request could not be read, so no approval was read."
  fi

  while IFS=$'\t' read -r login id; do
    [[ -n "${login}" ]] || continue
    # The login goes into an API path below, so this case checks it first.
    case "${login}" in
      *[!A-Za-z0-9-]*)
        refuse "A login is written into an Accepted-by trailer only when it holds letters, digits \
and hyphens, because the trailer names the person who accepts the commit, and GitHub allows \
no other character in a person's login. Ask a person to approve the pull request." \
          "login: ${login}"
        finish
        ;;
      *) ;;
    esac
    case "${id}" in
      "" | *[!0-9]*)
        refuse "An id is written into an Accepted-by trailer only when it is a number, because the \
trailer reaches main, where no commit message is edited, and GitHub gives each account a \
numeric id. Ask a person to approve the pull request." \
          "id: ${id:-none}"
        finish
        ;;
      *) ;;
    esac

    # GitHub reports the maintain role as write, and the triage role as read,
    # under "Get repository permissions for a user":
    #   https://docs.github.com/en/rest/collaborators/collaborators
    if ! answer=$(gh api "repos/${GH_REPO}/collaborators/${login}/permission") ||
      ! permission=$(jq -er '.permission' <<<"${answer}"); then
      cannot "the permission of ${login} could not be read, so no approver was named."
    fi
    if [[ "${permission}" == admin || "${permission}" == write ]]; then
      printf '%s %s\n' "${login}" "${id}"
      return 0
    fi
  done <<<"${standing}"
  return 0
}
