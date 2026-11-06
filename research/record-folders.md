# What does a folder for each kind bring beyond finding a record, and is a folder with an index worth its cost?

Measured on `main` at `142a236`. The 95 records are the same 95 that the layout research sorted, so its kinds apply unchanged.

Besides findability, a folder brings one benefit that purba can use today.
Git and GitHub can name a kind by its path, so `git log`, a code owner and a code search can each take one folder.
History by folder starts at the move, though: each folder shows 1 commit, and the 39 to 81 commits before the move stay on the old paths.
A kind line gives the same history today, with no move, because the paths stay where they are.
A code owner for each kind holds only with a second human, and purba has one.
The agent benefit does not hold.
`mise run list:records` already hands an agent the records of the kind it works in, by reach and not by kind.
`AGENTS.md` forbids a link to a folder, because each line links "down to the section of a record".
No reach, gate, workflow or code-owner line lists the records of one kind, so a folder glob replaces no list.
No template part and no gate rule differs by kind today.

An index page sinks as the folder grows.
On the flat folder, GitHub renders `README.md` below 97 rows, and records arrive at about 11 a week.
With five folders, the listing is 7 rows, so a `README.md` renders near the top.
That fact holds today, and it holds for the five-folder level only.
The largest folder, one section of the product design, already lists 35 records.

The move is cheaper in the tree than the layout research suggested, and dearer off the tree.
A 33-line script rewrites every in-tree link, and `lint:links` then refuses 1 link, a `blob/main` address that stays dead until the move merges.
The tooling needs 6 changed lines in 3 scripts and 10 changed patterns.
Then 40 of 129 bats tests fail, because their fixtures build a flat folder.
The probe found one more mechanism that breaks with no refusal.
After the move, the list of records on every pull request drops from 17 to 2, and the command exits 0.
Off the tree, 1,624 `blob/main` links to the current records sit in 849 issue bodies, pull-request bodies and comments, and each one returns 404.
GitHub does not redirect a moved path, and the probe shows the 404.
Of those links, 327 are in 101 merged pull-request bodies, and `AGENTS.md` says "Never edit the merged description".

```text
 WHAT A FOLDER GIVES, BESIDE A KIND LINE                    holds today?   only a folder gives it?
 ─────────────────────────────────────────────────────────────────────────────────────────────────
 folder page shows the kind (findability, out of scope)     yes            yes; a generated index also does
 an agent reads one folder in place of 95                   NO             -   list:records already does it, by reach
 a reach / gate / workflow names one folder                 NO             -   no list of one kind exists to replace
 a template or a gate rule for one kind                     NO             -   no rule differs by kind; a line feeds a gate too
 a code owner for one kind                                  NO, 1 human    yes, or a filename prefix (CODEOWNERS reads paths)
 git log / GitHub history for one kind                      partly         no: a kind line gives 39-81 commits, a folder 1
 a smaller git tree for each commit                         NO, 7.5 KB     -   GitHub's limit is 3,000 entries a directory
 ─────────────────────────────────────────────────────────────────────────────────────────────────

 WHERE A README.md RENDERS ON GITHUB
   flat, today              docs/decisions/   ███████████████████████████████████████████████  97 rows, then README
   folders, top level       docs/decisions/   ███▌                                               7 rows, then README
   folders, largest folder  one-section/      █████████████████▌                                36 rows, then README
                            (each █ is 2 rows; each count includes the README row)
```

## 1. What a folder brings besides findability

Each candidate is measured on the tree, or rejected with its reason.
The last column says whether a kind line in each record, with no move, gives the same thing.

| candidate | measured | holds? | a kind line gives it too? |
|---|---|---|---|
| an agent reads one folder in place of 95 records | `AGENTS.md` cites 18 records in 39 links; its stages do cluster by kind; but `list:records` already names 13 of the 14 "how a record is written" records for a record edit, and 31 of 35 section records for a crate edit | no | not needed |
| a reach, gate or workflow names one folder in place of a list | 9 reaches name `docs/decisions/*.md`, meaning every record; 1 names one record; no file lists the records of one kind | no | not needed |
| a template or a gate rule for one kind | the template has 0 kind words; `lint:records` has 19 rules, 0 by kind | no | yes, a gate can read a line |
| a code owner for one kind | `.github/CODEOWNERS` names 1 human, `@snregales`, on 5 lines; the repository has 1 human and 1 agent account | no, until a second human | no; a filename prefix can |
| `git log` and history for one kind | after the move, each folder shows 1 commit; before it, the kinds' records hold 39 to 81 commits | only from the move on | yes, with the whole history |
| a smaller git tree | the flat tree object is 7,473 bytes; GitHub's documented limit is 3,000 entries | no | not needed |

### An agent reads one folder in place of 95 records

**What the stages cite.** `AGENTS.md` cites 18 distinct records in 39 links.
Of the 18, 9 are "how a record is written", 7 are pull-request workflow, and 2 are repository tooling.
None is product design or one section.
The stages do cluster by kind.
"When you write a record" cites 6 records, and all 6 are "how a record is written".
"In anything you write" cites 3, all of the same kind.
The pull-request stages cite workflow records, with one tooling record each in two stages.

| file | links | distinct records | their kinds |
|---|---:|---:|---|
| `AGENTS.md` | 39 | 18 | 9 how a record is written, 7 workflow, 2 tooling |
| `docs/agents/post-pass.md` | 6 | 5 | 3 how a record is written, 2 workflow |
| `CONTRIBUTING.md` | 3 | 3 | 2 how a record is written, 1 tooling |
| `.github/PULL_REQUEST_TEMPLATE.md` | 2 | 2 | 1 of each |
| each of the 4 issue templates | 1 | 1 | how a record is written |
| `README.md`, `CLAUDE.md` | 0 | 0 | |

**Why it does not hold.** Three facts reject it.

- **The register forbids the link.** `AGENTS.md` admits a line only when "the line links that file, down to the section of a record". A link to a folder links no section.
- **A folder adds what the stage does not cite.** The "how a record is written" folder holds 14 records. "When you write a record" cites 6 of them, so a folder link hands the agent 8 more.
- **The reach already does it.** A one-line edit to a record makes `list:records` name or list 26 records. Those 26 hold 13 of the 14 "how a record is written" records. A one-line edit to `crates/assertion/src/lib.rs` names 40 records: 31 of the 35 section records, 5 product design and 4 tooling. The 4 section records it leaves out reach `tests/**`, `crates/command/tests/**` or only `/LANDSCAPE.md` and `/CONTEXT.md`, so each is left out by its own reach.

A folder would hand an agent records by kind.
The reach hands it records by the location it changes, and that list crosses kinds.

### A reach, a gate or a workflow names one folder

No list of one kind exists, so a folder glob replaces nothing.

| location | what it names today | after a move |
|---|---|---|
| 9 reaches | `docs/decisions/*.md`, every record | `docs/decisions/**/*.md`, every record; the meaning stays "all" |
| 1 reach | `docs/decisions/a-design-in-depth-waits-for-what-it-assumes.md`, one record | one record at a new path |
| `.github/CODEOWNERS` | `/docs/decisions/` | covers subfolders already |
| `.config/readers` | `docs/decisions/**` | covers subfolders already |
| `.config/gates` | `docs/**` | covers subfolders already |
| workflow `paths:` | `cargo-scripts.yml` only, with no record path; `build.yml`, `quality.yml` and `scripts.yml` say "Never add a `paths:` filter here" | nothing to name |

A future reach could name one kind, such as "every tooling record".
No record on `main` asks for one.

### A template or a records-gate rule for one kind

No rule differs by kind today.
The template mentions no kind and no section of `LANDSCAPE.md`.
`lint:records` reads 19 tagged rules, and each one applies to every record.

The only feature I found that follows a kind is a link to `LANDSCAPE.md`.

| kind | records | mention `LANDSCAPE.md` |
|---|---:|---:|
| one section of the product design | 35 | 20 |
| product design, whole | 12 | 10 |
| repository tooling | 24 | 1 |
| how a record is written | 14 | 1 |
| pull-request workflow | 10 | 0 |

A rule "a section record links its section" would refuse 15 records today.
`LANDSCAPE.md` also has no heading for each part, so a record cannot link its part's own section.
A gate can read a kind line as well as a path, so this rule needs the kind written down, and not a folder.

The other sets show the same split.
Kubernetes' KEP-0000 says SIGs "have the freedom to customize the KEP template according to their SIG-specific concerns".
Swift keeps three templates and one extra folder: the package-manager template has no folder of its own, and its proposals sit in `proposals/` "for historical reasons".

### A code owner for one kind

This is the benefit that a folder gives and a kind line does not.
`CODEOWNERS` matches a path, with gitignore rules, so it cannot read a line inside a file.
A filename prefix such as `docs/decisions/tooling-*` matches too.

It does not hold for purba today.
`CODEOWNERS` names one owner, `@snregales`, on all 5 of its lines.
The repository has two collaborators: `snregales` (admin) and `snregales-agent` (write).

Kubernetes is the set where it holds.
23 of the 25 folders under `keps/` hold an `OWNERS` file, and `sig-node/OWNERS` names `sig-node-tech-leads` and the label `sig/node`.
KEP-0000 gives this as the reason for a folder: "KEPs in SIG specific subdirectories have limited impact outside of the SIG and can leverage SIG-specific OWNERS files."
Swift's `proposals/testing/` belongs to the Testing Workgroup, a different decider from the rest of `proposals/`.

### `git log` and history for one kind

History by folder starts at the move.
Before the move, git knows each record by its flat path.

| kind | commits that touch its records on `main` | `git log -- <folder>/` on `prototype/801-folder-per-kind` |
|---|---:|---:|
| how a record is written | 81 | 1 |
| one section of the product design | 80 | 1 |
| repository tooling | 79 | 1 |
| pull-request workflow | 73 | 1 |
| product design, whole | 39 | 1 |

`git log --follow` recovers one file's history, 29 commits for `a-gate-owns-the-mechanical`.
It takes one file only, and on a folder it still prints 1 commit.
GitHub's commits API gives 1 commit for the folder and 1 for a moved file.
On `prototype/801-kind-line-and-index`, the 24 records whose line opens with "Repository tooling" give 80 commits, with no move.

### Others I checked

- **A smaller git tree.** GitHub's repository limits page recommends "Segmenting files into a shallow directory structure" and sets 3,000 entries for each directory. The flat tree object is 7,473 bytes. The five folder trees are 750 to 2,889 bytes, plus 266 for the parent. At 11 records a week, the flat folder reaches 3,000 in about five years.
- **"Typos can easily be spotted".** MADR's ADR-0010 gives this as a pro of subfolders, because a wrong category shows as a new folder. A closed list in a gate refuses a wrong kind word too.
- **A record beside its code.** log4brains keeps "package-specific ADRs" in `packages/<package name>/docs/adr`. That folder follows a location, not a kind. purba keeps the location in each reach.
- **The claim the large sets make for folders.** MADR: "grouping is done by folders (which are natural for grouping)". Kubernetes: the OWNERS files above. Swift: one decider, one template and one number prefix, ST. The KEP README says where a KEP goes, and gives no reason. Of the 4 sets I read, the reasons are findability, a separate owner, a separate decider, and the location of the code.

## 2. Does an index page stay findable as the folder grows?

### How fast records arrive

| ISO week | records added |
|---|---:|
| 2026-W37 | 9 |
| 2026-W38 | 8 |
| 2026-W39 | 2 |
| 2026-W40 | 19 |
| 2026-W41 | 10 |
| 2026-W42 | 20 |
| 2026-W43 | 4 |
| 2026-W44 | 19 |
| 2026-W45 | 8 |

98 records were added from 2026-09-11 to 2026-11-05, and 3 were deleted.
The mean is about 11 a week, and a week ranges from 2 to 20.
The history also holds 4 renames.

If the mean holds, the flat folder lists about 180 records at the end of 2026, and about 670 a year from now.
The section kind holds 35 of 95, about 37%. At that share it lists about 250 in a year.
These are arithmetic on a nine-week mean, not a forecast.

### How GitHub shows a README.md below a list

I fetched the folder page of `prototype/801-kind-line-and-index` and read its HTML.

- **The page holds every row, then the README.** It names `totalCount: 97`. The 95 records, `.template.md` and `README.md` all appear as rows. The rendered README starts after the last row, at byte 496,939 of 548,672.
- **The README has an anchor.** The container carries `id="readme"`, so a link to the folder page with `#readme` can point at it. I did not open it in a browser.
- **A hidden row says "View all files".** It carries the class `d-none`. I did not see what shows it.
- **A subfolder renders its README too.** GitHub's API "Get a repository README for a directory" returns `docs/decisions/README.md` on the prototype branch, and `keps/README.md` for Kubernetes. It returns 404 for `docs/decisions` on `main`, which has none.

On `prototype/801-folder-per-kind`, the folder page names `totalCount: 6`: the five folders and `.template.md`.
A `README.md` there is row 7, then it renders.
That confirms the lead's claim for the top level.
Inside a folder the list is long again: one section holds 35, tooling 24, how a record is written 14, product design 12, workflow 10.

### Where else an index can live

| where | rows in the listing, its own included | code owner today | `lint:records` |
|---|---|---|---|
| `docs/decisions/README.md`, flat | 97 | yes, `/docs/decisions/` | refused: `is_record` reads it as a record |
| `docs/decisions/README.md`, with folders | 7 | yes | refused, until the scripts leave it out |
| `docs/decisions/<kind>/README.md` | 11 to 36 | yes | refused, once `is_record` reads subfolders |
| `docs/README.md` | 3: `agents/`, `decisions/` and itself | no: `CODEOWNERS` has no line for it | not read |
| a link in the root `README.md` | root listing | no | not read; `README.md` links no record today |

A `docs/README.md` does not replace the repository's README.
GitHub's docs say it picks `.github`, then the root, then `docs`, and the root has one.
One `CODEOWNERS` line, `/docs/README.md @snregales`, would give it the owner.
`a-record-names-every-location-where-it-applies` rejected `.config/records` because "It is an index, and a change to it escapes the code owner".

## 3. What a folder plus an index costs, beside each form alone

### What a script can rewrite

I ran a probe on a worktree of `prototype/801-folder-per-kind`, in a fresh clone.
A 33-line Bash and Perl script rewrote every path the move breaks.
It leaves a permalink at a commit, `blob/<sha>/...`, as it is, because the old path is right at that commit.

| step | what changed | what the gate said |
|---|---|---|
| the move alone | 95 renames | `lint:links` refuses 361 links and 96 comment paths |
| the script | 432 lines in 138 files: 74 records and 64 other files | `lint:links` refuses 1 link |
| the one left | `.github/scripts/post-record-thread.sh` L61 links `blob/main/docs/decisions/how-a-record-is-written/...` | a `blob/main` link to the new path is dead until the move merges |
| the move, before any code change | | `lint:records` exits 2: "no record found in docs/decisions." |
| 4 code lines | `check-records.sh` L25 glob, `records.sh` L29 `is_record`, `list-records.sh` L103 `ls-tree -r`, L169 link regex | `lint:records` refuses 30 records for their reach |
| 1 more line | `records.sh` L90: the reach grammar names `../../CONTEXT.md` | `lint:records` exits 0 |
| 10 patterns | 9 reaches and `post-pass.md` L10: `docs/decisions/*.md` to `docs/decisions/**/*.md` | |
| the bats tests | none changed | 40 of 129 fail in 4 files: 39 in `list-records.bats`, 1 in `records.bats` |

**A new silent break.** `records.sh` L130 also names `../../CONTEXT.md`, inside `reach_is_everywhere`.
Left as it is, a record whose reach holds a glossary word drops off the list for every pull request.
I made a one-line change after the move and ran `list:records`.
It listed 2 records on every pull request, against 17 on `main`, and exited 0.
With L130 fixed, it listed 17.
The layout research found seven mechanisms that skip a subfolder with no refusal. This is an eighth.

**Why the bats tests fail.** `list-records.bats` L17 builds `a-record-is-rewritten-not-amended` at the flat path, and the script now looks in the folder.
`records.bats` test 1 pins "a Markdown file directly under docs/decisions/ is a record, and nothing else is".
I traced the cause of 2 of the 40 failures, not each one.

### What a script cannot rewrite

| where | `blob/main` links to a current record | can it change? |
|---|---:|---|
| issue bodies | 259 in 118 issues | the API can edit them; `AGENTS.md` allows "a typo or a wrong filename" on an issue with a type |
| merged pull-request bodies | 327 in 101 | `AGENTS.md`: "Never edit the merged description." |
| open pull-request bodies | 5 in 3 | yes |
| issue and pull-request comments | 999 in 575 | the API can edit them; edit history stays |
| review comments | 53 in 52 | the API can edit them |
| **total** | **1,624, in 849 items** | |

Of 1,643 `blob/main` record links, 1,624 point at the 95 current records.
The other 19 point at records that are already gone.
I counted bodies, issue comments and review comments.
I did not count review summaries, discussions or links from outside GitHub.
Commit messages on `main` cite 0 record paths.
A permalink at a commit, `blob/<sha>/...`, stays valid. There are 11,377 of them, mostly in comments.

**The redirect, re-probed.** On `prototype/801-folder-per-kind`, the old path of `a-gate-owns-the-mechanical` returns 404, and the new path returns 200.
`raw.githubusercontent.com` returns 404 for the old path too.
`blob/main/src/lib.rs` returns 404, and its new path `crates/purba/src/lib.rs` returns 200.
A stub at each old path would keep the links, but it keeps 95 files in the flat listing, and `is_record` reads each one as a record.

**A later kind change is a new move.** A record whose kind changes moves folders, and its links break again.
A kind line changes one line, and no link breaks.
Swift's process says "Proposal documents should generally not be renamed once they've been added to the repository."

### How the large sets keep folder and index true

| set | folder | line beside it | what keeps them true | measured |
|---|---|---|---|---|
| Kubernetes KEPs | `keps/<sig>/` | `owning-sig` and `participating-sigs` in `kep.yaml` | `kepctl create` writes the folder from `owning-sig`; `api/proposal.go` refuses an unknown SIG | 8 of 649 KEPs sit in a folder that differs from `owning-sig`; 6 of them are in `provider-aws/` |
| Python PEPs | none, flat | `Type` and `Topic` headers | `check-peps.py`; the index is generated | from the layout research |
| Swift | `proposals/` and `proposals/testing/` | none; the prefix is `SE` or `ST` | the workgroup | not measured |
| MADR | recommends one level of subfolders | none | its ADR-0010 lists "two indices have to be maintained" as a con | its own log is flat |

In the KEP code I read, nothing compares the folder with `owning-sig` after creation.
`pkg/repo/write.go` builds the path as `filepath.Join(r.ProposalPath, sig, kepName)`.
306 of 649 KEPs name a participating SIG other than the owner, so about half are of two kinds, and the folder holds only one.

purba has a model for a generated list that stays true.
`fmt:order` rewrites the order section of `LANDSCAPE.md`, and `lint:order` runs the same script with `--check` and refuses a difference.
An index could follow it: one task writes the page, and one refuses a stale page.

### Where a record of two kinds goes

A folder holds a record once, so each of the 12 records of two kinds sits in its primary folder.
An index can list it under both kinds only when the second kind is written down.
So a folder plus an index of two kinds needs a line in those 12 records, or a hand-written list.
Kubernetes chose the first: the folder for the owner, and `participating-sigs` in the file.

### One index at the top, or one in each folder

| | one `docs/decisions/README.md` | one `README.md` in each folder |
|---|---|---|
| files to keep true | 1 | 5, or 13 with a folder for each section |
| rows in the listing, its own included | 7 | 11 to 36 |
| what it adds to GitHub's listing | the kind's name, the full title, the two-kind records, the sections | the same, inside one kind |
| a record added | 1 line in it | 1 line in its folder's file |

Generated, the two cost one script either way, and the check reads 1 file or 5.
Each folder page already lists its own records.
83 of 95 filenames are the slug of their title, so the listing already shows most titles.
A page in each folder adds the 12 full titles that differ, the two-kind records, and the sections.

### The forms side by side

| | kind line | index page, flat | folder | folder plus index |
|---|---|---|---|---|
| files moved | 0 | 0 | 95 | 95 |
| in-tree lines rewritten by a script | 0 | 0 | 432 in 138 files | 432 in 138 files |
| code lines in scripts | the gate, for the line | 2 scripts exclude `README.md` | 6 lines in 3 scripts | 6 lines, and `README.md` left out at two levels |
| bats tests that fail before fixtures change | 0 | `records.bats` | 40 of 129 | 40 of 129, and more for `README.md` |
| `blob/main` links that return 404 | 0 | 0 | 1,624 | 1,624 |
| mechanisms that break with no refusal | 0 | 0 | 8 | 8 |
| history for one kind | whole | whole | from the move | from the move |
| code owner for one kind | no | no | yes | yes |
| rows in the listing, the README included | | 97 | | 7 |
| a two-kind record | named twice | listed twice | one folder | one folder, and a line to list it twice |
| decided text it contradicts | none | "there is no index" | the path in `a-record-is-rewritten-not-amended`, `.template.md` L2 | both |

`a-record-is-rewritten-not-amended` says "Filenames carry no numbers and there is no index."
Its Considered Options weigh no index.
It chose a convention that "has no runtime, so nothing can go dead".
A generated index with a check is a runtime of that kind.

## Red team

| finding | what would make it wrong | what I did |
|---|---|---|
| the agent benefit does not hold | an agent reads records outside `list:records` and `AGENTS.md` | I measured the two routes `AGENTS.md` and `post-pass.md` give. A session that greps on its own is not measured. |
| `list:records` names 13 of 14 for a record edit | the edited record changes the list | I edited `purba-is-always-strict`, a section record. Another record names its own extra links too. |
| no list of one kind exists | a list hides in a comment | I read every reach, `CODEOWNERS`, `.config/readers`, `.config/gates` and every workflow `paths:`. I did not read every script comment. |
| history by folder starts at the move | GitHub's web page follows renames | the API returns 1 commit for the folder and for one file. I did not open the web page in a browser. |
| a script fixes the in-tree links | the script writes a wrong but live link | `lint:links` proves each target exists, not that it is the right one. Permalinks did not change: 0 changed lines hold `blob/<sha>/docs/decisions/<folder>/`. I read a sample of the diff. |
| an eighth silent mechanism | the drop is an artefact of the probe | the control is the same tree with L130 fixed: 2 records against 17. `main` lists 17. |
| 1,624 links return 404 | GitHub redirects for some readers | I probed `github.com` and `raw.githubusercontent.com` for one moved record, and `src/lib.rs` on `main`. All three return 404. |
| 8 of 649 KEPs disagree | my parse of `owning-sig` misses a quoted form | I stripped quotes and read the first `owning-sig:` line of each `kep.yaml` at `92eb8a1`. |
| 306 KEPs of two kinds | `owning-sig` comes after `participating-sigs` in some files | my awk assumes the template's order. |
| about 11 records a week | the first weeks ported records in bulk | the weeks range from 2 to 20. The projection is arithmetic only. |

## What I could not measure

- How GitHub's page looks in a browser: the scroll depth in pixels, the "View all files" row, and the `#readme` jump.
- Links to records from outside GitHub, such as search engines, chats and other repositories.
- PR review summaries and discussions, which I did not count.
- How a person searches for a record in practice, which the person knows.
- How the Kubernetes KEP site and the Swift dashboard build their lists.

## Sources

- Kubernetes KEP README: https://github.com/kubernetes/enhancements/blob/master/keps/README.md
- KEP-0000, "Git and GitHub Implementation" and the SIG template sentence: https://github.com/kubernetes/enhancements/blob/master/keps/sig-architecture/0000-kep-process/README.md
- A SIG's OWNERS file: https://github.com/kubernetes/enhancements/blob/master/keps/sig-node/OWNERS
- KEP path from `owning-sig`: https://github.com/kubernetes/enhancements/blob/master/pkg/repo/write.go
- KEP SIG validation: https://github.com/kubernetes/enhancements/blob/master/api/proposal.go
- MADR ADR-0010: https://github.com/adr/madr/blob/develop/docs/decisions/0010-support-categories.md
- log4brains README, "What about multi-package projects?": https://github.com/thomvaill/log4brains/blob/develop/README.md
- Swift evolution areas and the rename rule: https://github.com/swiftlang/swift-evolution/blob/main/process.md
- Swift testing template: https://github.com/swiftlang/swift-evolution/blob/main/proposal-templates/0000-swift-testing-template.md
- GitHub, About READMEs: https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes
- GitHub, About code owners: https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners
- GitHub, Repository limits: https://docs.github.com/en/repositories/creating-and-managing-repositories/repository-limits
- GitHub API, a README for a directory: https://docs.github.com/rest/repos/contents#get-a-repository-readme-for-a-directory
- The layout research, for the kinds and the costs it measured: `research/record-layout.md` on `research/record-layout`

## Appendix: how each number was measured

All on `142a236` in a fresh clone, unless the row says so. `$P` is a prototype branch.

| number | command |
|---|---|
| each record's kind | the appendix of `research/record-layout.md`, parsed to a table |
| citations in `AGENTS.md` and the other files | `grep -oE 'decisions/[a-z0-9-]+\.md' <file>`, each joined to its kind |
| records `list:records` names | a one-line commit in a worktree, then `scripts/list-records.sh HEAD~1`, reading "Named by this change" and "On every pull request" |
| reaches and `LANDSCAPE.md` mentions by kind | `reach_patterns` from `scripts/records.sh` for each record, and `grep -c LANDSCAPE.md` |
| 19 gate rules | `grep -cE '^  [a-z]+ (refuse\|report) "' scripts/check-records.sh` |
| code owners and collaborators | `.github/CODEOWNERS`; `gh api repos/kalonji-tools/purba/collaborators` |
| commits by kind | `git log --oneline origin/main -- <the kind's records>`, against `git log --oneline $P -- docs/decisions/<folder>/` |
| one file with `--follow` | `git log --follow --oneline $P -- docs/decisions/repository-tooling/a-gate-owns-the-mechanical.md` |
| GitHub commits for a folder | `gh api 'repos/kalonji-tools/purba/commits?sha=prototype/801-folder-per-kind&path=docs/decisions/repository-tooling'` |
| tree sizes | `git cat-file -s <rev>:docs/decisions` and each subfolder |
| records added by week | `git log origin/main --diff-filter=A --format=@%cs --name-only -- 'docs/decisions/*.md'`, less `.template.md`, then `date +%G-W%V` |
| deletions and renames | the same with `--diff-filter=D`, and with `-M --diff-filter=R` |
| the GitHub folder page | `curl -sL https://github.com/kalonji-tools/purba/tree/$P/docs/decisions`, read for `totalCount`, `title="…md"`, `readmeContainer` and `view-all-files-row` |
| README for a directory | `gh api 'repos/<owner>/<repo>/readme/<dir>?ref=<ref>'` |
| `blob/main` links | `gh api --paginate` on `issues?state=all`, `issues/comments` and `pulls/comments`, matched for `kalonji-tools/purba/blob/main/docs/decisions/<slug>.md`, then joined to the 95 records |
| links by issue or merged pull request | the same issues listing, split by `pull_request.merged_at` |
| the 404 probe | `curl -s -o /dev/null -w '%{http_code}' -L <url>` |
| the rewrite probe | the move from `$P`, then the script, then `mise exec -- scripts/check-links.sh` with `PURBA_BASE=142a236`, `scripts/check-records.sh`, `scripts/list-records.sh`, and `bats` on `records`, `list-records`, `check-records` and `post-record-thread` |
| titles that differ from the filename | each `# ` title, lowercased and slugged, compared with the filename |
| KEP folders and `owning-sig` | a sparse clone of `kubernetes/enhancements` at `92eb8a1`, reading `keps/*/*/kep.yaml` |
