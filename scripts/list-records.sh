#!/usr/bin/env bash
# The records a change must satisfy, in Markdown for the post-pass comment.
#
#   the decision:  docs/decisions/a-record-names-every-location-where-it-applies.md
#   the task:      mise run list:records
#
#   list-records.sh [--added <pattern>] [<base>]
#
# Reads the change from where HEAD left <base> to HEAD. <base> defaults to
# `origin/main`.
#
#   on every pull request      its reach holds `**` or a glossary word
#   named by this change       a pattern in its reach matches a changed path, a
#                              changed file outside docs/decisions/ links it at
#                              the base, or the change adds, edits or deletes it
#   may hold a stale sentence  it links an issue a commit subject names, or a
#                              line outside its reach names a changed file's
#                              basename or parent directory, in a record the
#                              change does not edit
#
# With --added, it prints `<path>:<line>: <text>` for each line the change adds
# in a file the gitattributes <pattern> matches, and nothing else.
#
# Exits 0 whenever it reads the change, and 2 when it cannot. It never refuses.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"
# shellcheck source=scripts/reach.sh
. "$(dirname "$0")/reach.sh"

added="" given=origin/main
while [[ $# -gt 0 ]]; do
  if [[ "$1" == --added ]]; then
    [[ -n "${2:-}" ]] || cannot '--added needs a gitattributes pattern.'
    added=$2
    shift 2
  else
    given=$1
    shift
  fi
done

# git names a changed path from the top of the repository, and a pathspec from
# where it runs.
top=$(git rev-parse --show-toplevel 2>/dev/null) ||
  cannot 'the records can only be read inside a git repository.'
cd "${top}"
base=$(git merge-base "${given}" HEAD 2>/dev/null) ||
  cannot "git cannot read the base ${given}."
head=$(git rev-parse HEAD)

scratch=$(mktemp -d)
trap 'rm -rf "${scratch}"' EXIT

git diff -z --name-only --no-renames "${base}" HEAD >"${scratch}/changed" ||
  cannot 'git cannot list the files the change touches.'

if [[ -n "${added}" ]]; then
  printf '%s added\n' "${added}" >"${scratch}/pattern"
  reach_match "${scratch}" "${scratch}/pattern" "${scratch}/changed" >"${scratch}/hits"
  # The flags, and `GIT_DIFF_OPTS` unset, fix what a user setting can change: the
  # hunk context, the algorithm, the indent heuristic, colour, an external tool
  # and a textconv driver. The path comes from the list of changed files, because git quotes a
  # path in a diff header.
  while IFS= read -r -d '' _ && IFS= read -r -d '' file; do
    env -u GIT_DIFF_OPTS git diff -U0 --inter-hunk-context=0 --diff-algorithm=default \
      --indent-heuristic --no-renames --no-color --no-ext-diff --no-textconv \
      "${base}" HEAD -- ":(literal)${file}" |
      file=${file} awk '
        /^@@ / { split($3, at, ","); line = substr(at[1], 2) + 0; hunk = 1; next }
        hunk && /^\+/ { print ENVIRON["file"] ":" line ": " substr($0, 2); line++ }
      '
  done <"${scratch}/hits"
  exit 0
fi

remote=$(git remote get-url origin 2>/dev/null) ||
  cannot 'the address of the repository is read from the origin remote, and there is none.'
# A URL opens with a scheme, and can carry a user and a port. The form scp
# writes has no scheme and no port, and its host ends at a colon.
if [[ "${remote}" =~ ^[a-z+]+://([^@/]+@)?([^/:]+)(:[0-9]+)?/(.+)$ ]]; then
  host=${BASH_REMATCH[2]} repository=${BASH_REMATCH[4]}
elif [[ "${remote}" =~ ^([^@/]+@)?([^/:]+):(.+)$ ]]; then
  host=${BASH_REMATCH[2]} repository=${BASH_REMATCH[3]}
else
  cannot "the origin remote ${remote} names no repository address."
fi
repository=${repository%/}
address="https://${host}/${repository%.git}"

rewrite=docs/decisions/a-record-is-rewritten-not-amended.md
git cat-file -e "HEAD:${rewrite}" 2>/dev/null ||
  cannot "the record a stale sentence breaks is not at the head: ${rewrite}"

mapfile -d '' -t changed <"${scratch}/changed"
declare -A edited=()
for path in "${changed[@]}"; do
  edited[${path}]=1
done

# A record is a Markdown file directly under docs/decisions/, and a leading dot
# keeps the template out.
is_record='^docs/decisions/[^/.][^/]*\.md$'
listing=$(git ls-tree --name-only "${base}" docs/decisions/) ||
  cannot "git cannot list the records at the base."
mapfile -t at_base <<<"${listing}"
records=()
for path in "${at_base[@]}" "${changed[@]}"; do
  [[ ! "${path}" =~ ${is_record} ]] || records+=("${path}")
done
sorted=$(printf '%s\n' "${records[@]}" | LC_ALL=C sort -u)
mapfile -t records <<<"${sorted}"

# A record the change deletes is read and linked at the base.
declare -A text=() commit=()
mkdir "${scratch}/records"
for i in "${!records[@]}"; do
  path=${records[i]}
  commit[${path}]=${head}
  read_at=${base}
  if [[ -n "${edited[${path}]:-}" ]]; then
    if git cat-file -e "HEAD:${path}" 2>/dev/null; then
      read_at=${head}
    else
      commit[${path}]=${base}
    fi
  fi
  text[${path}]=${scratch}/records/${i}
  git cat-file blob "${read_at}:${path}" >"${text[${path}]}" ||
    cannot "git cannot read ${path} at ${read_at}."
done

link() {
  local title
  title=$(sed -n '/^# /{s///p;q}' "${text[$1]}")
  printf '[%s](%s/blob/%s/%s)' "${title:-$1}" "${address}" "${commit[$1]}" "$1"
}

# Each reach is one attribute, so a match names its record.
declare -A everywhere=() named=() reach_at=() reach_lines=()
: >"${scratch}/reaches"
for i in "${!records[@]}"; do
  path=${records[i]}
  paragraph=$(reach_paragraph <"${text[${path}]}")
  reach=${paragraph#*$'\n'}
  [[ "${reach}" == '**Reach:**'* ]] || continue
  reach_at[${path}]=${paragraph%%$'\n'*}
  reach_lines[${path}]=$(wc -l <<<"${reach}")
  [[ "${reach}" != *'](../../CONTEXT.md#'* ]] || everywhere[${path}]=1
  while [[ "${reach}" =~ \`([^\`]+)\` ]]; do
    [[ "${BASH_REMATCH[1]}" != '**' ]] || everywhere[${path}]=1
    printf '%s r%d\n' "${BASH_REMATCH[1]}" "${i}" >>"${scratch}/reaches"
    reach=${reach#*"${BASH_REMATCH[0]}"}
  done
done

reach_match "${scratch}" "${scratch}/reaches" "${scratch}/changed" >"${scratch}/matched"
while IFS= read -r -d '' attribute && IFS= read -r -d '' _; do
  named[${records[${attribute#r}]}]=1
done <"${scratch}/matched"

for path in "${changed[@]}"; do
  if [[ -n "${text[${path}]:-}" ]]; then
    named[${path}]=1
    continue
  fi
  # A link from a record is the record's own business, and a file the change
  # adds linked nothing at the base.
  [[ "${path}" != docs/decisions/* ]] || continue
  # A path the change turns from a directory into a file was a tree at the base.
  type=$(git cat-file -t "${base}:${path}" 2>/dev/null) || continue
  [[ "${type}" == blob ]] || continue
  git cat-file blob "${base}:${path}" >"${scratch}/file"
  while read -r linked; do
    linked=docs/${linked}
    [[ -z "${text[${linked}]:-}" ]] || named[${linked}]=1
  done < <(grep -IoE 'decisions/[a-z0-9][a-z0-9-]*\.md' "${scratch}/file" | sort -u || true)
done

log=$(git log --format=%s "${base}..HEAD") ||
  cannot "git cannot read the commit subjects of the change."
mapfile -t subjects <<<"${log}"
declare -A issues=()
for subject in "${subjects[@]}"; do
  while [[ "${subject}" =~ \#([0-9]+) ]]; do
    issues[${BASH_REMATCH[1]}]=1
    subject=${subject#*"${BASH_REMATCH[0]}"}
  done
done
: >"${scratch}/names"
for path in "${changed[@]}"; do
  # A name that holds a newline cannot sit inside one line of a record.
  [[ "${path}" != *$'\n'* ]] || continue
  printf '%s\n' "${path##*/}" >>"${scratch}/names"
  while [[ "${path}" == */* ]]; do
    path=${path%/*}
    printf '%s/\n' "${path}" >>"${scratch}/names"
  done
done
sort -u -o "${scratch}/names" "${scratch}/names"

stale=()
for path in "${records[@]}"; do
  while read -r number; do
    [[ -z "${issues[${number}]:-}" ]] ||
      stale+=("- \`${path}\` links #${number}, which a commit subject names")
  done < <(grep -o "${address}/issues/[0-9][0-9]*" "${text[${path}]}" |
    sed 's|.*/||' | sort -un || true)
done
for path in "${records[@]}"; do
  [[ -z "${edited[${path}]:-}" ]] || continue
  first=${reach_at[${path}]:-0}
  last=$((first + ${reach_lines[${path}]:-0}))
  lines=$(awk -v first="${first}" -v last="${last}" -v path="${path}" '
    # A directory counts only where a path starts with it, after any `./` or
    # `../`, so `scripts/` is not named by `.github/scripts/`.
    function holds(line, name, rest, before, start, at) {
      if (name !~ /\/$/) return index(line, name)
      rest = line
      while ((at = index(rest, name)) > 0) {
        before = before substr(rest, 1, at - 1)
        start = before
        sub(/(\.\.?\/)+$/, "", start)
        sub(/\/$/, "", start)
        if (start !~ /[[:alnum:]._-]$/) return 1
        before = before substr(rest, at, 1)
        rest = substr(rest, at + 1)
      }
      return 0
    }
    NR == FNR { names[NR] = $0; count = NR; next }
    FNR >= first && FNR < last { next }
    {
      for (n = 1; n <= count; n++)
        if (holds($0, names[n])) printf "- `%s:%d` names `%s`\n", path, FNR, names[n]
    }
  ' "${scratch}/names" "${text[${path}]}")
  [[ -z "${lines}" ]] || mapfile -t -O "${#stale[@]}" stale <<<"${lines}"
done

every=()
second=()
for path in "${records[@]}"; do
  if [[ -n "${everywhere[${path}]:-}" ]]; then
    every+=("$(link "${path}")")
  elif [[ -n "${named[${path}]:-}" ]]; then
    second+=("- $(link "${path}")")
  fi
done

short_base=$(git rev-parse --short "${base}")
short_head=$(git rev-parse --short HEAD)
printf "Base \`%s\`, head \`%s\`.\n\n" "${short_base}" "${short_head}"

if [[ ${#every[@]} -eq 0 ]]; then
  printf '**On every pull request:** none.\n\n'
else
  joined=$(printf '%s, ' "${every[@]}")
  printf '**On every pull request:** %s\n\n' "${joined%, }"
fi

if [[ ${#second[@]} -eq 0 ]]; then
  printf '**Named by this change:** none.\n\n'
else
  printf '**Named by this change:**\n'
  printf '%s\n' "${second[@]}"
  printf '\n'
fi

heading=$(link "${rewrite}")
if [[ ${#stale[@]} -eq 0 ]]; then
  printf '**May hold a stale sentence:** (%s) none.\n' "${heading}"
else
  printf '**May hold a stale sentence:** (%s)\n' "${heading}"
  printf '%s\n' "${stale[@]}"
fi
