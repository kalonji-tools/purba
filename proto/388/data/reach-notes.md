# Reach notes

Written from the 34 record texts, `CONTEXT.md` and `tree_271base.txt` only. No issue, pull request, git history or other file was read. `union.txt` sits in the folder. It was not named as an input, so it was not opened.

Conventions:

- A record's own file is left out of its reach, because every record binds itself. The one exception is `an-actor-is-what-it-does-not-what-it-is.md`. The readers check reads that record's roster rows as data.
- A gate's script is in the reach when the record names it as what holds the rule. Its `.bats` test is not, unless the record names the test.
- Gitattributes `*` does not treat a leading dot as special. So `docs/decisions/*.md` also matches `docs/decisions/.template.md`, and the records that bind the template rely on that.
- `Issue` covers an issue's body, title, type and comments. `Pull request` covers a pull request's body, reviews and comments. A narrower term (`Design spec`, `Implementation plan`, `Post pass`, `Debrief`) is added only where the record gives that sub-location a rule of its own.

## (a) Glossary words used

| word | records |
|---|---:|
| Pull request | 6 |
| Issue | 4 |
| Commit message | 4 |
| Design spec | 2 |
| Implementation plan | 2 |
| Debrief | 2 |
| Unpublished comment | 2 |
| Post pass | 1 |
| Published comment | 1 |
| Outside contribution | 1 |

`Unpublished comment` stands for a comment that can sit in any non-Markdown file. It is used for that record and for the inline exclusions in `a-gate-owns-the-mechanical`. `Record` was never used as a word, because `docs/decisions/*.md` is its path.

## (b) Records where `**` was the honest answer

- **`a-location-inherits-its-readers.md`.** A tracked file that reaches no actor fails the build. So adding a file at any path can break the record. Only `.readers` is bound for its content. Gitattributes patterns cannot tell an addition from an edit, so `**` is the only pattern that covers additions.
- **`an-artifact-holds-the-minimum-that-conveys-its-point.md`.** The record says "Provenance crosses every location", and an artifact is anything purba writes for a reader. Its decidable rules cover records only, but the sufficiency rule covers everything. The reach is `**` plus `Commit message`, `Issue` and `Pull request`.

## (c) Hard choices

- **`a-gate-owns-the-mechanical.md` (16 patterns, the widest list).** "A configuration file is owed only where purba deviates from a tool's default" is broken by adding a file. The file need not exist yet. So the reach names `rustfmt.toml`, `clippy.toml` and the `typos` configuration names, and none of them is in the tree. "Every lint level lives in `Cargo.toml`" is broken by a flag in any workflow, so `.github/workflows/*.yml` is in. The record does not name `deny.toml` or `lychee.toml`. They are in because the strictest-setting rule binds every detecting tool. An inline exclusion can sit next to any line of code, so it takes `Unpublished comment`.
- **`a-term-belongs-to-the-glossary.md`.** "A term is defined once, in `CONTEXT.md`" could read as binding every prose file, because a definition in `README.md` would also break it. The decision and its options compare only records with the glossary, so the reach stays at `/CONTEXT.md docs/decisions/*.md`.
- **`a-standard-a-gate-cannot-decide-does-not-become-a-gate.md`.** The record binds every place a refusal can be wired. A judgement turns into a gate when a gate script refuses on it. For example, `check-records.sh` could start to refuse the passive voice. So gate scripts and tasks are in, beside the workflows and the ruleset. `prek.toml` is out, because a hook refuses at the commit and a gate refuses a merge.
- **`a-register-belongs-to-one-location.md`.** The closed locations are bound in one way only: one must not open without a reader and a register. `CHANGELOG.md` and `cliff.toml` are named as paths, and neither exists. The wiki, discussions and docs site have no path and no term.
- **`liability-is-recorded-from-the-act-that-makes-it-true.md`.** `.github/scripts/write-acceptance-trailer.sh` is not named in the record. It is included because the record says a workflow writes the acceptance trailer, and the tree has exactly one file for that act. The record also mentions an advisory origin check that runs on every push. That check's workflow is not named, so `commits.yml` was left out.
- **`an-artifact-is-rewritten-until-its-direction-is-agreed.md`.** The record names `sign.yml` as the file that posts the review thread. `.github/scripts/post-record-thread.sh` probably does the posting, but the record does not name it, so it is out.
- **`only-github-runs-what-lives-under-github.md`.** A caller "is added by an edit somewhere else". Any file could name a script, for example `.config/wt.toml`. The reach names only the callers the record names: workflows, `tasks.toml`, `prek.toml` and both script directories.
- **`a-normalisers-style-is-chosen-for-the-human-who-reads-the-code.md`.** The two-space rule touches the indentation of every file. The record decides the style, not the files, and `editorconfig-checker` enforces it on files. So the reach is `/.editorconfig` alone.
- **`purba-records-delegation-and-does-not-prevent-it.md`.** This record is close to a pure ruling. Its only file is `scripts/sign-branch.sh`, which must not claim more than it checks.
- **Two near-duplicate records.** `mise-names-every-tool-version.md` and `purba-carries-no-tool-manager-beside-mise-and-no-compiler-of-its-own.md` share most of their text. The first says each workflow names mise, so it takes `.github/workflows/*.yml`. The second names only `build.yml`. Both take `devenv.*`, because a second manager's file would break "no second environment manager".
- **`significance-is-declared-not-detected.md` and `the-person-owns-the-decision-outcome.md`.** Both include `docs/decisions/*.md`. A change there is what triggers the routing each record decides. The routing itself can break in `CODEOWNERS` and in the ruleset.

## (d) Locations CONTEXT.md has no term for

| location | records |
|---|---|
| ruleset (`protect-main`: required reviews, `bypass_actors`, thread resolution) | 5: outside-contribution, standard-a-gate-cannot-decide, liability, significance, person-owns |
| repository setting (Actions may create and approve pull requests; `web_commit_signoff_required`) | 2: workflow-identity, liability |
| GitHub App (installation and permissions of `purba-bump`) | 1: workflow-identity |
| wiki | 1: register |
| discussions | 1: register |
| public docs site | 1: register |

The ruleset is the largest gap. Five records are broken by a change that is in no file, no commit and no pull request.
