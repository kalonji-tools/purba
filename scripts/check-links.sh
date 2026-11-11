#!/usr/bin/env bash
# The links that do not reach their target as written, and the paths a comment
# cites that git does not track.
#
#   the decision:  docs/decisions/an-unpublished-comment-carries-what-no-other-location-carries.md
#   the task:      mise run lint:links
#   the settings:  pyproject.toml, under `[tool.lychee]`
#
#   check-links.sh
#
#   PURBA_BASE   the commit this branch is compared with. Without it, the merge
#                base with origin/main
#
# Exits 1 when it refuses a link or a path, and 2 when it cannot run.
#
# It reports every refusal before it exits, because an author repairing one
# should not have to run it again to find the next.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"

top=$(git rev-parse --show-toplevel 2>/dev/null) ||
  cannot 'the links can only be read inside a git repository.'
cd "${top}"

# A file that .gitattributes marks generated holds what a tool wrote, so no
# author chose an address in it. Every read below leaves it out.
authored=':(exclude,attr:linguist-generated)'

written=$(git -c core.quotePath=false ls-files -- . "${authored}") ||
  cannot 'the tracked files cannot be listed.'

base=${PURBA_BASE:-}
if [[ -z "${base}" ]]; then
  base=$(git merge-base origin/main HEAD 2>/dev/null) ||
    cannot 'the base cannot be found. Fetch origin/main, or name the base in PURBA_BASE.'
fi

# The comparison is with the working tree, so an edit is read before it is
# committed. A base that starts with a dash is still a revision, never an option.
changed=$(git -c core.quotePath=false diff --name-only --no-renames --diff-filter=d \
  --end-of-options "${base}" -- . "${authored}") ||
  cannot "what this branch changes against ${base} cannot be read."

inputs=""
while IFS= read -r file; do
  [[ -n "${file}" && -f "${file}" ]] || continue
  inputs+="${inputs:+$'\n'}${file}"
done <<<"${written}"

# links <summary> <files> [<flag>...]
#
# Reports the dead links lychee finds, and warns of each link whose server does
# not answer, each path made relative to the top. It stops the check when
# lychee cannot run. lychee exits 2 for a dead link and for
# a mistake in how it was called, so a dead link is told apart by the line it
# prints. In plain mode that line starts with `[`. The colour mode, lychee's
# default whenever FORCE_COLOR turns colour on, can put spaces before it. The
# address below cannot be wrapped, so the style check skips that line.
# editorconfig-checker-disable-next-line
# https://github.com/lycheeverse/lychee/blob/lychee-v0.24.2/lychee-bin/src/formatters/response/color.rs#L44
links() {
  local summary=$1 files=$2 found status=0
  shift 2
  found=$(lychee --no-progress --mode plain --format compact "$@" --files-from - \
    <<<"${files}" 2>&1) || status=$?
  [[ ${status} -ne 0 ]] || return 0
  found=${found//"file://${top}/"/}
  if [[ ${status} -eq 2 ]] && grep -q '^\[' <<<"${found}"; then
    sort_found "${found}"
    [[ -z "${unanswered}" ]] || warn "A link whose server does not answer in time, or \
answers 503, passes unchecked, because the state of that server is no fault of this change." \
      "${unanswered}"
    [[ -z "${dead}" ]] || refuse "${summary}" "${dead}"
    return 0
  fi
  cannot 'lychee could not run.' "${found}"
}

# sort_found <lychee's lines>
#
# Sets `unanswered` to each link that timed out or answered 503, for the warning,
# and `dead` to each other line that lychee starts with `[`, for the refusal.
# Each link sits under the line that names its file, and a file is named in each
# list that holds a link of it.
sort_found() {
  local line file="" to
  local -A named=()
  unanswered=""
  dead=""
  while IFS= read -r line; do
    case "${line}" in
      \[*\]:) file=${line} named=() ;;
      \[TIMEOUT\]\ * | \[503\]\ *) to=unanswered ;;
      \[*) to=dead ;;
      *) continue ;;
    esac
    [[ "${line}" != "${file}" ]] || continue
    if [[ -n "${file}" && -z "${named[${to}]:-}" ]]; then
      printf -v "${to}" '%s%s' "${!to:+${!to}$'\n'}" "${file}"
      named[${to}]=1
    fi
    printf -v "${to}" '%s%s' "${!to:+${!to}$'\n'}" "${line}"
  done <<<"$1"
}

links "A link to a file or a heading in this tree is refused when its target does not \
exist. A renamed file leaves every link to it dead." "${inputs}" --offline

# The offline leg reads every link to a file already, so this one reads only an
# address on the network, and never reports a dead file link a second time.
if [[ -n "${changed}" ]]; then
  links "A link in a file this branch changes is refused unless it reaches its target as written, \
or its server does not answer in time or answers 503, because its reader must reach the address \
it names. A link in any other file is read only for a target in this tree." "${changed}" \
    --scheme https \
    --scheme http
fi

# PyPI's renderer points a link to a heading at the same page, so that one passes.
# https://github.com/pypa/readme_renderer/blob/main/readme_renderer/markdown.py
readme=$(sed -n 's/^readme = "\(.*\)"$/\1/p' pyproject.toml 2>/dev/null || true)
[[ -n "${readme}" ]] ||
  cannot 'pyproject.toml names no readme, so the page PyPI shows cannot be found.'
found=$(lychee --dump --offline --files-from - <<<"${readme}" 2>&1) ||
  cannot 'lychee could not run.' "${found}"
relative=()
while IFS= read -r address; do
  [[ "${address}" == file://* && "${address}" != "file://${top}/${readme}#"* ]] || continue
  relative+=("${readme}: ${address#"file://${top}/"}")
done <<<"${found}"

[[ ${#relative[@]} -eq 0 ]] || refuse \
  "A link in the README is refused unless it names a full address or a heading of the \
README, because PyPI shows the README as purba's page, and any other link resolves to nothing \
there." \
  "${relative[@]}"

# A comment line opens with `#` or `//`. Markdown is left out, because `#` opens
# a heading there.
comments=$(git -c core.quotePath=false grep -n -I -E '^[[:space:]]*(#|//)' -- \
  ':!*.md' "${authored}") || {
  status=$?
  [[ ${status} -eq 1 ]] ||
    cannot 'the comments cannot be read.'
  comments=""
}

# A cited path starts at one of the directories a reader opens, and does not end
# on the full stop of the sentence it sits in.
roots='crates|docs|scripts|src|\.github|\.config'
cites="(^|[^A-Za-z0-9_./-])(${roots})/[A-Za-z0-9_./*-]*[A-Za-z0-9_*/-]"
missing=()
while IFS= read -r hit; do
  [[ -n "${hit}" ]] || continue
  file=${hit%%:*}
  rest=${hit#*:}
  line=${rest%%:*}
  tokens=$(grep -oE "${cites}" <<<"${rest#*:}" || true)
  while IFS= read -r path; do
    [[ -n "${path}" ]] || continue
    [[ "${path}" =~ ^(${roots})/ ]] || path=${path:1}
    path=${path%/}
    [[ "${path}" != *'*'* ]] || continue
    # A cited directory passes when git tracks a file inside it.
    git ls-files --error-unmatch -- "${path}" >/dev/null 2>&1 ||
      missing+=("${file}:${line}: ${path}")
  done <<<"${tokens}"
done <<<"${comments}"

[[ ${#missing[@]} -eq 0 ]] || refuse \
  "A path that a comment cites is refused when git does not track it. A renamed file \
leaves every comment that cites it pointing nowhere." \
  "${missing[@]}"

finish
