# shellcheck shell=bash
# Which approval on a pull request an Accepted-by trailer is written from.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#
#   standing_approver
#       reads the reviews of one pull request on standard input, and prints
#       `<login> <id>` of the approval that stands, or nothing when none stands
#
# A login or an id that no trailer may hold exits 1, and reviews the rule cannot
# read exit 2.

# shellcheck source=scripts/report.sh
. "$(dirname "${BASH_SOURCE[0]}")/../../scripts/report.sh"

standing_approver() {
  local reviews stands approver approver_id
  reviews=$(cat)
  if ! jq -e 'type == "array"' >/dev/null 2>&1 <<<"${reviews}"; then
    cannot "the reviews of this pull request are not a JSON array, so no approval was read."
  fi
  if ! stands=$(jq -c '
    to_entries | map(.value + {order: .key})
    | map(select(.state == "APPROVED" or .state == "CHANGES_REQUESTED" or .state == "DISMISSED"))
    | group_by(.user.id) | map(max_by(.order)) | map(select(.state == "APPROVED"))
    | max_by(.order) // empty' <<<"${reviews}"); then
    cannot "the reviews of this pull request could not be read, so no approval was read."
  fi
  approver=$(jq -r '.user.login // empty' <<<"${stands}")
  approver_id=$(jq -r '.user.id // empty' <<<"${stands}")
  [[ -n "${approver}" ]] || return 0

  case "${approver}" in
    *[!A-Za-z0-9-]*)
      refuse "A login is written into an Accepted-by trailer only when it holds letters, digits \
and hyphens, because the trailer names the person who accepts the commit, and GitHub allows \
no other character in a person's login. Ask a person to approve the pull request." \
        "login: ${approver}"
      finish
      ;;
    *) ;;
  esac
  case "${approver_id}" in
    "" | *[!0-9]*)
      refuse "An id is written into an Accepted-by trailer only when it is a number, because the \
trailer reaches main, where no commit message is edited, and GitHub gives each account a \
numeric id. Ask a person to approve the pull request." \
        "id: ${approver_id:-none}"
      finish
      ;;
    *) ;;
  esac

  printf '%s %s\n' "${approver}" "${approver_id}"
}
