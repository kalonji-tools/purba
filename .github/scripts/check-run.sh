# shellcheck shell=bash
# What every caller of the check-runs API reads and writes.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#   who reports:   docs/decisions/a-required-check-of-main-counts-only-from-github-actions.md
#   which counts:  docs/decisions/a-gate-counts-only-by-a-token-github-signed-for-its-run.md
#
#   verdicts <head> <context>...   prints `<context> <conclusion>` for each, in
#                                  the order given, and `none` for no conclusion
#   require_green <head> <context>...
#                                  returns 0 when every context is green, and 1
#                                  when one is not. With no context it exits 2
#   post_check_run <name> <head> <conclusion> <title> <summary>
#
#   GH_REPO        the repository the check runs belong to. A caller that
#                  sources this file without it exits 2
#   HEAD_REF       the branch of the pull request. A read with none exits 2
#   PR, BASE_REF   the number and the base of that pull request, which the token
#                  of a gate must name. A read with either empty exits 2
#   SIGN_TOKEN     the token of `purba-sign`, which posts each check run. A
#                  write with none exits 2
#
# A read or a write the API refuses exits 2, because the caller cannot run.
if [[ -z "${GH_REPO:-}" ]]; then
  echo "GH_REPO is set by the workflow env, and it is empty here." >&2
  exit 2
fi

# The check runs of a head hold only the newest run of each name, so a later
# post would hide a job there. So this function reads the verdict from the
# job itself.
#
# The shape is read here, once, so `last_conclusion` cannot fail on it.
check_runs_of() {
  local runs made jobs='[]' ids one id query listed
  if [[ -z "${HEAD_REF:-}" ]]; then
    echo "HEAD_REF is set by the workflow env, and it is empty here." >&2
    exit 2
  fi
  if [[ -z "${PR:-}" ]]; then
    echo "PR is set by write-acceptance-trailer.sh, and it is empty here." >&2
    exit 2
  fi
  if [[ -z "${BASE_REF:-}" ]]; then
    echo "BASE_REF is set by the workflow env, and it is empty here." >&2
    exit 2
  fi
  if ! runs=$(gh api "repos/${GH_REPO}/commits/$1/check-runs?per_page=100") ||
    ! jq -e '.check_runs | type == "array" and all(.[]; type == "object")' \
      >/dev/null <<<"${runs}"; then
    echo "the check runs on $1 could not be read." >&2
    exit 2
  fi
  query="head_sha=$1&event=pull_request&branch=${HEAD_REF}&per_page=100"
  if ! made=$(gh api "repos/${GH_REPO}/actions/runs?${query}") ||
    ! jq -e '.workflow_runs | type == "array" and all(.[]; type == "object")' \
      >/dev/null <<<"${made}"; then
    echo "the workflow runs on $1 could not be read." >&2
    exit 2
  fi
  ids=$(jq -r --arg branch "${HEAD_REF}" '.workflow_runs[]
    | select(.event == "pull_request" and .head_branch == $branch) | .id' <<<"${made}")
  # Each payload reaches jq on standard input, never as an argument. Linux
  # refuses one argument of 32 pages or more, counting the null byte that ends
  # it: 131071 bytes is the most with pages of 4 KiB. The check runs of a head
  # with many jobs grow past that.
  # https://man7.org/linux/man-pages/man2/execve.2.html
  # `filter=latest` reads the last attempt of a run, so a rerun replaces it.
  while read -r id; do
    [[ -n "${id}" ]] || continue
    if ! one=$(gh api "repos/${GH_REPO}/actions/runs/${id}/jobs?filter=latest&per_page=100") ||
      ! jq -e '.jobs | type == "array" and all(.[]; type == "object")' >/dev/null <<<"${one}"; then
      echo "the jobs of the workflow run ${id} could not be read." >&2
      exit 2
    fi
    jobs=$(printf '%s\n%s' "${jobs}" "${one}" | jq -sc '.[0] + .[1].jobs')
  done <<<"${ids}"
  listed=$(printf '%s\n%s\n%s' "${runs}" "${made}" "${jobs}" | jq -s --arg branch "${HEAD_REF}" '
    .[0] as $runs | .[1] as $made | .[2] as $jobs
    | [$made.workflow_runs[]
      | select(.event == "pull_request" and .head_branch == $branch)
      | {key: (.id | tostring), value: .path}]
    | from_entries as $file
    | {check_runs: ([$runs.check_runs[] | select(.app.slug == "purba-sign")]
        + [$jobs[]
          | select(.runner_id != null
              and $file[(.run_id | tostring)]
                == ".github/workflows/\(.name | split("-")[0] | ascii_downcase).yml")])}')
  proved "${listed}"
}

#   proved <listed>   prints <listed> with each job that no token proves taken out
proved() {
  local keys lines kept='[]' run attempt job token
  lines=$(jq -r '.check_runs[] | select(.run_id != null)
    | "\(.run_id) \(.run_attempt) \(.id)"' <<<"$1")
  if [[ -n "${lines}" ]]; then
    if ! keys=$(curl --silent --show-error --fail --location \
      https://token.actions.githubusercontent.com/.well-known/jwks) ||
      ! jq -e '.keys | type == "array" and all(.[]; type == "object")' \
        >/dev/null <<<"${keys}"; then
      echo "the key set of GitHub could not be read." >&2
      exit 2
    fi
  fi
  while read -r run attempt job; do
    [[ -n "${run}" ]] || continue
    token=$(token_of "${run}" "${attempt}") || exit 2
    # shellcheck disable=SC2310 # `token_holds` returns 1 for a token that does not hold
    if [[ -n "${token}" ]] && token_holds "${token}" "${keys}" "${run}" "${attempt}" "${job}"; then
      kept=$(jq -c --argjson job "${job}" '. + [$job]' <<<"${kept}")
    fi
  done <<<"${lines}"
  jq --argjson kept "${kept}" '.check_runs |= map(select(.run_id == null
    or (.id as $id | $kept | index($id) != null)))' <<<"$1"
}

# Prints the token in the artifact that `CI` uploads for one attempt of its run.
# It prints nothing when the run holds no such artifact, or when that artifact expired.
#
#   token_of <run> <attempt>
token_of() {
  local listed id zip query="name=gate-token-$2&per_page=100"
  if ! listed=$(gh api "repos/${GH_REPO}/actions/runs/$1/artifacts?${query}") ||
    ! id=$(jq -er --arg name "gate-token-$2" '[.artifacts[]
      | select(.name == $name and .expired == false)][0].id // ""' <<<"${listed}"); then
    echo "the token of the workflow run $1 could not be read." >&2
    return 2
  fi
  [[ -n "${id}" ]] || return 0
  zip=$(mktemp)
  if ! gh api "repos/${GH_REPO}/actions/artifacts/${id}/zip" >"${zip}" ||
    ! unzip -p "${zip}" gate-token; then
    rm -f "${zip}"
    echo "the token of the workflow run $1 could not be read." >&2
    return 2
  fi
  rm -f "${zip}"
}

# Returns 0 when GitHub's key signed <token>, and each of its claims holds. The
# record that the header of this file names under "which counts" lists them.
# https://docs.github.com/en/actions/reference/security/oidc
#
#   token_holds <token> <keys> <run> <attempt> <job>
token_holds() {
  local header payload signature head kid modulus dir claims status=0
  IFS=. read -r header payload signature <<<"$1"
  [[ -n "${header}" && -n "${payload}" && -n "${signature}" ]] || return 1
  head=$(from_b64url "${header}") || return 1
  kid=$(jq -r 'select(.alg == "RS256") | .kid // empty' <<<"${head}" 2>/dev/null) || return 1
  [[ -n "${kid}" ]] || return 1
  modulus=$(jq -r --arg kid "${kid}" '[.keys[]
    | select(.kid == $kid and .kty == "RSA" and .e == "AQAB")][0].n // empty' <<<"$2")
  [[ -n "${modulus}" ]] || return 1
  dir=$(mktemp -d)
  if ! key_pem "${modulus}" "${dir}" ||
    ! from_b64url "${signature}" >"${dir}/signature"; then
    rm -r "${dir}"
    return 1
  fi
  printf '%s.%s' "${header}" "${payload}" >"${dir}/signed"
  openssl dgst -sha256 -verify "${dir}/key.pem" -signature "${dir}/signature" \
    "${dir}/signed" >/dev/null 2>&1 || status=1
  rm -r "${dir}"
  [[ "${status}" -eq 0 ]] || return 1
  claims=$(from_b64url "${payload}") || return 1
  jq -e --arg repo "${GH_REPO}" --arg ref "refs/pull/${PR}/merge" --arg base "${BASE_REF}" \
    --arg run "$3" --arg attempt "$4" --arg job "$5" '
      .iss == "https://token.actions.githubusercontent.com" and .aud == "purba-gate"
      and .repository == $repo and .event_name == "pull_request"
      and .ref == $ref and .base_ref == $base
      and (.run_id | tostring) == $run and (.run_attempt | tostring) == $attempt
      and (.check_run_id | tostring) == $job' >/dev/null 2>&1 <<<"${claims}"
}

# Writes the public key whose modulus is <n>, in base64url, to <dir>/key.pem.
# One key of GitHub's set carries no certificate, so this function builds each
# key from a DER header. The header fits RSA of 2048 bits with the exponent
# 65537, and `openssl` refuses a modulus of any other length.
# https://www.rfc-editor.org/rfc/rfc3279.html#section-2.3.1
#
#   key_pem <n> <dir>
key_pem() {
  local hex escaped="" i
  from_b64url "$1" >"$2/modulus" || return 1
  hex=$(od -An -v -tx1 "$2/modulus")
  hex=${hex//[$' \n']/}
  hex="30820122300d06092a864886f70d01010105000382010f003082010a0282010100${hex}0203010001"
  for ((i = 0; i < ${#hex}; i += 2)); do escaped+="\\x${hex:i:2}"; done
  printf '%b' "${escaped}" >"$2/key.der"
  openssl pkey -pubin -inform DER -in "$2/key.der" -out "$2/key.pem" 2>/dev/null
}

from_b64url() {
  local encoded=${1//_//}
  encoded=${encoded//-/+}
  while ((${#encoded} % 4)); do encoded+='='; done
  base64 -d <<<"${encoded}" 2>/dev/null
}

# The API promises no order, so the run that finished last is read rather than
# the one that happens to be last.
last_conclusion() {
  jq -r --arg c "$1" '
    [.check_runs[] | select(.name == $c)]
    | sort_by(.completed_at) | last | .conclusion // empty'
}

# A caller behind `||` or `if` turns `set -e` off inside a function, so each
# read that fails exits here, or the function would go on with no payload.
verdicts() {
  local runs context conclusion
  runs=$(check_runs_of "$1") || exit 2
  shift
  for context in "$@"; do
    conclusion=$(last_conclusion "${context}" <<<"${runs}")
    echo "${context} ${conclusion:-none}"
  done
}

require_green() {
  local head=$1 lines context conclusion
  # With no context the loop below reads one empty line and returns 1, as if a
  # gate had failed. A list that names nothing is the caller's mistake.
  if [[ $# -lt 2 ]]; then
    echo "usage: require_green <head> <context>..." >&2
    exit 2
  fi
  shift
  lines=$(verdicts "${head}" "$@") || exit 2

  while read -r context conclusion; do
    case "${conclusion}" in
      success) echo "${context} is green on ${head}" ;;
      none)
        echo "${context} has no verdict on ${head} yet"
        return 1
        ;;
      *)
        echo "${context} concluded ${conclusion} on ${head}"
        return 1
        ;;
    esac
  done <<<"${lines}"

  echo "every gate named here is green on ${head}"
}

# `status` is always `completed`: purba writes a verdict or writes nothing.
post_check_run() {
  if [[ -z "${SIGN_TOKEN:-}" ]]; then
    echo "SIGN_TOKEN is set by the workflow env, and it is empty here." >&2
    exit 2
  fi
  GH_TOKEN="${SIGN_TOKEN}" gh api --method POST "repos/${GH_REPO}/check-runs" \
    -f name="$1" \
    -f head_sha="$2" \
    -f status=completed \
    -f conclusion="$3" \
    -f "output[title]=$4" \
    -f "output[summary]=$5" \
    --jq '.html_url' || {
    echo "the check run $1 could not be written onto $2." >&2
    exit 2
  }
}
