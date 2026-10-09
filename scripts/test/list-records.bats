# scripts/list-records.sh against the changes it names records for.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../list-records.sh"
  make_repo
  site=https://github.com/owner/repo
  git remote add origin "${site}.git"
  address="${site}/blob"
  every="**On every pull request:**"
  named="**Named by this change:**"
  stale="**May hold a stale sentence:**"
  # The record a stale sentence breaks, which heads the third list.
  record a-record-is-rewritten-not-amended "\`nowhere\`"
}

# A record called <name>, whose reach is <reach>, and then each <line>.
record() {
  local name=$1 reach=$2
  shift 2
  mkdir -p docs/decisions
  {
    printf '# The %s record\n\n## Decision Outcome\n\n**Reach:** %s\n\n**It holds.**\n' \
      "${name}" "${reach}"
    [[ $# -eq 0 ]] || printf '%s\n' "$@"
  } >"docs/decisions/${name}.md"
}

# Everything written so far is the base, at `origin/main`, and the change
# starts after it.
land_base() {
  git add --all
  git commit --quiet --message "chore: the base"
  git update-ref refs/remotes/origin/main HEAD
  git switch --quiet --create pr
}

# Everything written since is one commit of the change.
land_change() {
  git add --all
  git commit --quiet --message "${1:-feat: a change}"
}

# The paragraph of the output that opens with <label>, into `got`.
section() {
  got=$(awk -v label="$1" 'index($0, label) == 1 { at = 1 } at && !NF { exit } at' <<<"${output}")
}

# The link a list writes for the record called <name> at <commit>, into `want`.
link() {
  local commit
  commit=$(git rev-parse "${2:-HEAD}")
  printf -v want '[The %s record](%s/%s/docs/decisions/%s.md)' "$1" "${address}" "${commit}" "$1"
}

@test "a reach that holds a glossary word puts its record on every pull request" {
  record words "[commit message](../../CONTEXT.md#commit-message)"
  land_base
  echo more >other
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link words
  section "${every}"
  [[ "${got}" == *"${want}"* ]]
  section "${named}"
  [[ "${got}" != *"words record"* ]]
}

@test "a reach that holds ** puts its record on every pull request" {
  record all "\`**\`"
  land_base
  echo more >other
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link all
  section "${every}"
  [[ "${got}" == *"${want}"* ]]
}

@test "the records on every pull request share one line" {
  record words "[commit message](../../CONTEXT.md#commit-message)"
  record all "\`**\`"
  land_base
  echo more >other
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link all
  first=${want}
  link words
  section "${every}"
  [[ "${got}" == "${every} ${first}, ${want}" ]]
}

@test "a pattern that matches a changed path names its record" {
  record tools "\`tools/*.sh\`"
  land_base
  mkdir tools
  echo run >tools/a.sh
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link tools
  section "${named}"
  [[ "${got}" == *"- ${want}"* ]]
  section "${every}"
  [[ "${got}" != *"tools record"* ]]
}

@test "a pattern that matches no changed path leaves its record off every list" {
  record tools "\`tools/*.sh\`"
  land_base
  echo more >other
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"tools record"* ]]
  section "${named}"
  [[ "${got}" == "${named} none." ]]
}

@test "a pattern is matched as gitattributes matches it, not as a pathspec" {
  record tools "\`tools/*.sh\`"
  land_base
  mkdir -p tools/sub
  echo run >tools/sub/a.sh
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"tools record"* ]]
}

@test "a changed file outside docs/decisions that links a record at the base names it" {
  record linked "\`nowhere\`"
  echo "see docs/decisions/linked.md" >notes
  land_base
  echo more >>notes
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link linked
  section "${named}"
  [[ "${got}" == *"- ${want}"* ]]
}

@test "a link the change writes names nothing" {
  record linked "\`nowhere\`"
  echo notes >notes
  land_base
  echo "see docs/decisions/linked.md" >>notes
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"linked record"* ]]
}

@test "a link from a changed record names nothing but that record" {
  record linked "\`nowhere\`"
  record linking "\`nowhere\`" "It follows [the linked record](linked.md)."
  land_base
  echo "It holds still." >>docs/decisions/linking.md
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link linking
  section "${named}"
  [[ "${got}" == *"- ${want}"* ]]
  [[ "${output}" != *"linked record"* ]]
}

@test "a record the change adds is named" {
  land_base
  record added "\`nowhere\`"
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link added
  section "${named}"
  [[ "${got}" == *"- ${want}"* ]]
}

@test "the reach of a record the change edits is read at the head" {
  record edited "\`nowhere\`"
  land_base
  record edited "[commit message](../../CONTEXT.md#commit-message)"
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link edited
  section "${every}"
  [[ "${got}" == *"${want}"* ]]
}

@test "a record on every pull request is not named again" {
  record words "[commit message](../../CONTEXT.md#commit-message) \`tools/*.sh\`"
  land_base
  mkdir tools
  echo run >tools/a.sh
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link words
  section "${every}"
  [[ "${got}" == *"${want}"* ]]
  section "${named}"
  [[ "${got}" != *"words record"* ]]
}

@test "a record the change deletes is named, and linked at the base" {
  record deleted "\`nowhere\`"
  land_base
  git rm --quiet docs/decisions/deleted.md
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link deleted HEAD^
  section "${named}"
  [[ "${got}" == *"- ${want}"* ]]
}

@test "a record that links an issue a commit subject names may hold a stale sentence" {
  record waits "\`nowhere\`" "It waits on [a question](https://github.com/owner/repo/issues/12)."
  land_base
  echo more >other
  land_change "feat: a change (#12)"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link a-record-is-rewritten-not-amended
  section "${stale}"
  [[ "${got}" == "${stale} (${want})"$'\n'* ]]
  [[ "${got}" == *$'\n'"- \`docs/decisions/waits.md\` links #12, which a commit subject names"* ]]
}

@test "an issue whose number only starts the same is not a stale sentence" {
  record waits "\`nowhere\`" "It waits on [a question](https://github.com/owner/repo/issues/12)."
  land_base
  echo more >other
  land_change "feat: a change (#1)"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link a-record-is-rewritten-not-amended
  section "${stale}"
  [[ "${got}" == "${stale} (${want}) none." ]]
}

@test "a line that names a changed file's basename may hold a stale sentence" {
  record tools "\`nowhere\`" "\`a.sh\` runs the tools."
  mkdir tools
  echo run >tools/a.sh
  land_base
  echo more >>tools/a.sh
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  section "${stale}"
  [[ "${got}" == *$'\n'"- \`docs/decisions/tools.md:8\` names \`a.sh\`"* ]]
}

@test "a line that names a parent directory of a changed file may hold a stale sentence" {
  record tools "\`nowhere\`" "Every script in \`tools/\` is tested."
  mkdir -p tools/sub
  echo run >tools/sub/a.sh
  land_base
  echo more >>tools/sub/a.sh
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  section "${stale}"
  [[ "${got}" == *$'\n'"- \`docs/decisions/tools.md:8\` names \`tools/\`"* ]]
}

@test "a line in a record the change edits is not a stale sentence" {
  record tools "\`nowhere\`" "\`a.sh\` runs the tools."
  mkdir tools
  echo run >tools/a.sh
  land_base
  echo more >>tools/a.sh
  echo "It holds still." >>docs/decisions/tools.md
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  section "${stale}"
  [[ "${got}" != *"tools.md:"* ]]
}

@test "a reach is not a stale sentence" {
  record tools "\`tools/*.sh\`"
  mkdir tools
  echo run >tools/a.sh
  land_base
  echo more >>tools/a.sh
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  section "${stale}"
  [[ "${got}" != *"tools.md:"* ]]
}

@test "the output opens with the base and the head" {
  land_base
  echo more >other
  land_change
  base=$(git rev-parse --short HEAD^)
  head=$(git rev-parse --short HEAD)

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${lines[0]}" == "Base \`${base}\`, head \`${head}\`." ]]
}

@test "a remote written for ssh gives the same address" {
  record tools "\`tools/*.sh\`"
  git remote set-url origin git@github.com:owner/repo.git
  land_base
  mkdir tools
  echo run >tools/a.sh
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link tools
  section "${named}"
  [[ "${got}" == *"- ${want}"* ]]
}

@test "the base defaults to where the change left origin/main" {
  record tools "\`tools/*.sh\`"
  land_base
  echo more >other
  land_change
  git switch --quiet main
  mkdir tools
  echo run >tools/a.sh
  land_change "feat: main moves on"
  git update-ref refs/remotes/origin/main HEAD
  git switch --quiet pr

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"tools record"* ]]
}

@test "the base is an argument" {
  record tools "\`tools/*.sh\`"
  land_base
  mkdir tools
  echo run >tools/a.sh
  land_change
  echo more >other
  land_change

  run "${script}" HEAD^

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"tools record"* ]]
}

@test "outside a git repository it exits 2" {
  mkdir "${BATS_TEST_TMPDIR}/outside"
  cd "${BATS_TEST_TMPDIR}/outside"

  run --separate-stderr "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${stderr}" == *"git repository"* ]]
}

@test "a base git cannot read exits 2" {
  land_base

  run --separate-stderr "${script}" no-such-base

  [[ "${status}" -eq 2 ]]
  [[ "${stderr}" == *"no-such-base"* ]]
}

@test "a repository with no origin exits 2" {
  land_base
  git remote remove origin

  run --separate-stderr "${script}" main

  [[ "${status}" -eq 2 ]]
  [[ "${stderr}" == *"origin"* ]]
}

@test "a head with no record that a stale sentence breaks exits 2" {
  land_base
  git rm --quiet docs/decisions/a-record-is-rewritten-not-amended.md
  land_change

  run --separate-stderr "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${stderr}" == *"a-record-is-rewritten-not-amended.md"* ]]
}

@test "--added prints each line the change adds where the pattern matches" {
  record kept "\`nowhere\`"
  land_base
  echo "A new line." >>docs/decisions/kept.md
  mkdir tools
  echo run >tools/a.sh
  land_change

  run "${script}" --added 'docs/decisions/*.md'

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "docs/decisions/kept.md:8: A new line." ]]
}

@test "--added leaves out a line in docs/decisions/sub/x.md" {
  land_base
  mkdir -p docs/decisions/sub
  echo "A line below." >docs/decisions/sub/x.md
  land_change

  run "${script}" --added 'docs/decisions/*.md'

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "--added reads the change whatever diff the user configures" {
  record kept "\`nowhere\`"
  land_base
  echo "A new line." >>docs/decisions/kept.md
  land_change
  git config diff.noprefix true
  git config color.diff always

  run "${script}" --added 'docs/decisions/*.md'

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "docs/decisions/kept.md:8: A new line." ]]
}

@test "--added names a path that holds a space as git tracks it" {
  land_base
  mkdir "my docs"
  echo "A new line." >"my docs/a b.md"
  land_change

  run "${script}" --added '*.md'

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "my docs/a b.md:1: A new line." ]]
}

@test "--added after the base is read" {
  record kept "\`nowhere\`"
  land_base
  echo "A new line." >>docs/decisions/kept.md
  land_change

  run "${script}" main --added 'docs/decisions/*.md'

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "docs/decisions/kept.md:8: A new line." ]]
}

@test "run from a subdirectory, it reads the whole repository" {
  record tools "\`tools/*.sh\`"
  land_base
  mkdir tools
  echo run >tools/a.sh
  echo "A new line." >>docs/decisions/tools.md
  land_change
  cd tools

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link tools
  section "${named}"
  [[ "${got}" == *"- ${want}"* ]]

  run "${script}" --added 'docs/decisions/*.md'

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "docs/decisions/tools.md:8: A new line." ]]
}

@test "an ssh remote with a port and a trailing slash gives the same address" {
  record tools "\`tools/*.sh\`"
  git remote set-url origin ssh://git@github.com:22/owner/repo/
  land_base
  mkdir tools
  echo run >tools/a.sh
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link tools
  section "${named}"
  [[ "${got}" == *"- ${want}"* ]]
}

@test "--added counts lines as the change has them, whatever hunk context the user sets" {
  record kept "\`nowhere\`" "Six." "Seven." "Eight."
  land_base
  git config diff.interHunkContext 5
  sed -i -e 's/^Six\.$/Six.\nA first line./' -e 's/^Eight\.$/Eight.\nA second line./' \
    docs/decisions/kept.md
  land_change

  run "${script}" --added 'docs/decisions/*.md'

  [[ "${status}" -eq 0 ]]
  first="docs/decisions/kept.md:9: A first line."
  second="docs/decisions/kept.md:12: A second line."
  [[ "${output}" == "${first}"$'\n'"${second}" ]]
}

@test "--added prints a path that holds a quote as git tracks it" {
  land_base
  echo "A new line." >'q"x.md'
  land_change

  run "${script}" --added '*.md'

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == 'q"x.md:1: A new line.' ]]
}

@test "--added with no pattern exits 2" {
  land_base

  run --separate-stderr "${script}" --added ''

  [[ "${status}" -eq 2 ]]
  [[ "${stderr}" == *"--added"* ]]
}

@test "a change that replaces a directory with a file of its name is read" {
  record tools "\`tools\`"
  mkdir tools
  echo run >tools/a.sh
  land_base
  git rm --quiet -r tools
  echo run >tools
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link tools
  section "${named}"
  [[ "${got}" == *"- ${want}"* ]]
}

@test "a path that holds a newline is read" {
  record tools "\`tools/**\`"
  land_base
  mkdir tools
  echo run >"tools/a"$'\n'"b"
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link tools
  section "${named}"
  [[ "${got}" == *"- ${want}"* ]]
}

@test "an scp remote whose owner is a number is not read as a port" {
  record tools "\`tools/*.sh\`"
  owner=1234
  git remote set-url origin "git@github.com:${owner}/repo.git"
  address="https://github.com/${owner}/repo/blob"
  land_base
  mkdir tools
  echo run >tools/a.sh
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  link tools
  section "${named}"
  [[ "${got}" == *"- ${want}"* ]]
}

@test "a link to an issue page with no number is not an issue a subject names" {
  record form "\`nowhere\`" "It names [the form](${site}/issues/new/choose)."
  land_base
  echo more >other
  land_change "feat: a change (#12)"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"form.md"* ]]
}

@test "a directory is named only where a path starts with it" {
  record github "\`nowhere\`" \
    "Only \`.github/tools/x.sh\` runs there." "Every script in \`/tools/\` runs here."
  mkdir tools
  echo run >tools/a.sh
  land_base
  echo more >>tools/a.sh
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  section "${stale}"
  [[ "${got}" != *"github.md:8"* ]]
  [[ "${got}" == *"- \`docs/decisions/github.md:9\` names \`tools/\`"* ]]
}

@test "--added prints the same lines whatever diff algorithm the user sets" {
  mkdir -p docs/decisions
  printf '%s\n' 3 1 0 1 3 3 3 3 0 2 3 1 >docs/decisions/x.md
  land_base
  printf '%s\n' 3 1 3 0 0 3 1 0 3 1 0 2 >docs/decisions/x.md
  land_change
  run "${script}" --added 'docs/decisions/*.md'
  plain=${output}
  git config diff.algorithm histogram

  run "${script}" --added 'docs/decisions/*.md'

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "${plain}" ]]
}

@test "--added counts lines as the change has them, whatever GIT_DIFF_OPTS holds" {
  record kept "\`nowhere\`" "Six." "Seven." "Eight."
  land_base
  sed -i -e 's/^Six\.$/Six.\nA first line./' -e 's/^Eight\.$/Eight.\nA second line./' \
    docs/decisions/kept.md
  land_change

  GIT_DIFF_OPTS=--unified=3 run "${script}" --added 'docs/decisions/*.md'

  [[ "${status}" -eq 0 ]]
  first="docs/decisions/kept.md:9: A first line."
  second="docs/decisions/kept.md:12: A second line."
  [[ "${output}" == "${first}"$'\n'"${second}" ]]
}

@test "a relative path starts where its dots start" {
  record tools "\`nowhere\`" "Run [every script](../../tools/)." "Run \`./tools/x.sh\` first."
  mkdir tools
  echo run >tools/a.sh
  land_base
  echo more >>tools/a.sh
  land_change

  run "${script}"

  [[ "${status}" -eq 0 ]]
  section "${stale}"
  [[ "${got}" == *"- \`docs/decisions/tools.md:8\` names \`tools/\`"* ]]
  [[ "${got}" == *"- \`docs/decisions/tools.md:9\` names \`tools/\`"* ]]
}
