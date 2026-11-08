# CI and github.com while GitHub refuses each reader without a session

This file answers [How can CI check a link to github.com, and fetch what it downloads from there, while GitHub refuses each reader that is not signed in?](https://github.com/kalonji-tools/purba/issues/858)
It is never merged.

## Answer

During the outage, purba's link gate could not tell a live link from a dead one: both answered `504`.
Only one setting passes a live `/blob/` link and still refuses a dead one. It is a **remap** of `github.com/…/blob/…` to `raw.githubusercontent.com`.
Its cost is the words of [#523](https://github.com/kalonji-tools/purba/issues/523): the gate then checks an address other than the one written.
The GitHub token, retries, a cache, an accepted `504` and an exclude each fail a different way.
A finding that holds outside the outage: lychee 0.24.2 never checks a `#L` line anchor on a `/blob/` link. A built-in quirk deletes it before the request.

Other projects split in two. 9 of 20 link gates fetch github.com on each pull request and fail it, as purba does. 11 of 20 keep github.com off the blocking path.
GitHub publishes no limit and no status code for a reader of its web pages without a session, and its status page opened an incident only at 10:36Z, 1 h 45 min after the failures began.

The downloads failed on short retries: `setup-python` and CodeQL try 3 times, 10 to 20 s apart. Neither cached copy on the runner fits: it holds no free-threaded Python, and CodeQL 2.27.3 where the action wanted 2.27.4.
Only mise falls back to `api.github.com` by itself, and only for a lockfile row that records that URL.

| question | short answer | tier |
|---|---|---|
| 1. What lychee does with a github.com link and the token | fetches the page without a session; asks the API only after a failure, and the API answer counts only for a bare repo link or a private repo | [A] source + probes |
| 2. What lies between refuse and pass | only a remap to raw passes a live link and refuses a dead one; see the table | [A] probes on the live outage |
| 3. Other projects | 0 of 20 accept a 5xx; 0 of 20 exclude `/blob/`; 9 of 20 fail a pull request on github.com; 11 of 20 keep it off that path | [A] |
| 4. What GitHub documents | no limit and no status code for web pages; githubstatus.com opened an incident at 10:36Z | [A] |
| 5. The downloads | 3 tries, 10 to 20 s apart; the token does not change the URL; mise alone falls back to the API | [A] |

## 1. lychee, github.com and the token

mise resolves `lychee = "0.24"` to 0.24.2. The source below is at tag `lychee-v0.24.2`, commit `2bba271`.

```text
 a github.com link ──► GET the page, no session ──► 2xx? ── yes ──► pass
                                                     │ no
                                                     ▼
                       token set? ── no ──► keep the page's status (504)
                          │ yes
                          ▼
                       API: does owner/repo exist? ── private repo ──► pass (path never checked)
                          │ public
                          ▼
                       link has a path after owner/repo? ── yes ──► keep 504
                          │ no (bare repo link)
                          ▼
                         pass
```

| fact | source | tier |
|---|---|---|
| the API is asked only after the page fails | [`website.rs` L283-L295](https://github.com/lycheeverse/lychee/blob/lychee-v0.24.2/lychee-lib/src/checker/website.rs#L283-L295) | [A] |
| a public repo link with a path gets `InvalidGithubUrl`, so lychee keeps the `504` | [`website.rs` L308-L323](https://github.com/lycheeverse/lychee/blob/lychee-v0.24.2/lychee-lib/src/checker/website.rs#L308-L323) | [A] |
| a private repo passes for any path | the same, and its doc comment at L299-L306 | [A] |
| lychee retries a rejected status only when it is `429` | [`retry.rs` L87-L103](https://github.com/lycheeverse/lychee/blob/lychee-v0.24.2/lychee-lib/src/retry.rs#L87-L103) | [A] |
| a quirk deletes `#L…` from each `/blob/` link before the request | [`quirks/mod.rs` L23-L26, L95-L101](https://github.com/lycheeverse/lychee/blob/lychee-v0.24.2/lychee-lib/src/quirks/mod.rs#L95-L101) | [A] |
| a quirk sends `blob/…/*.md#heading` to raw, so that shape passed during the outage | [`quirks/mod.rs` L19-L22, L103-L114](https://github.com/lycheeverse/lychee/blob/lychee-v0.24.2/lychee-lib/src/quirks/mod.rs#L103-L114) | [A] |
| the retry fix for 5xx, PR #2324, merged 2026-10-21, ships only in `nightly` | [lychee#2324](https://github.com/lycheeverse/lychee/pull/2324); the newest tag is `lychee-v0.24.2` of 2026-05-01 | [A] |

purba's tree held 273 github.com links: 153 `/blob/` (126 with `#L`, 43 with `?plain=1`, 6 `.md#heading`), 2 `/tree/`, 101 issue or pull, 12 repo front pages.

## 2. Between refuse and pass

Each option ran on the live outage, 10:03Z to 10:22Z, with purba's `[tool.lychee]`. The baseline `/blob/` page answered `504` at the start and at the end.
Live links: L1 blob, L2 blob `#L`, L3 blob `?plain=1#L`, L4 blob `.md#heading`, L5 tree, L6 repo front page.
Dead links: D1 a missing path, D2 a missing commit, D3 a `#L` past the end, D4 a missing `.md` heading, D5 a missing directory, D6 a missing repo.

| option | live links that pass | dead links refused by GitHub's own answer | cost |
|---|---|---|---|
| no change | L4 | D4 | every other code page is refused while the outage lasts, dead or live |
| `GITHUB_TOKEN` | L6 | D4 | none beyond no change |
| `max_retries`, `retry_wait_time` | none | D4 | none, and no effect: lychee never retries a `504` |
| `accept` + `504` | all | D4 | a page that stays at `504` passes, dead or live |
| `exclude` github.com | all, unchecked | none | no github.com link is checked |
| `remap` blob → raw | L1 to L4 | D1, D2 (`404`), D4 | checks raw, not the address written; a renamed repo answers `200` on raw at its old path; the output names the raw URL |
| `remap` blob, tree → contents API | L1 to L5 | D1, D2, D5 (`404`) | D4 passes, since the body is JSON; drops `?plain=1`; 60 anonymous requests an hour |
| `cache` | a link with a stored success | none: a stored dead link passes | a link new to the branch gets nothing |
| `--header "Authorization: …"` | not shown: anonymous requests also got `200` at random | — | sends the token to each host lychee reads |
| browser `User-Agent`, `HEAD`, slower pacing, `--accept-timeouts` | none | — | none, and no effect |

D3 passes under every option, because lychee never checks a `#L` anchor.
The remap to raw, with purba's settings: `?plain=1` survives, raw sends no redirect, so `max_redirects = 0` holds. It cannot rewrite `/tree/`.
purba's exclude of fixture links (`//github\.com/(o|owner)/`) stops matching after a remap, so those fixture links would answer `404`.

**Re-run for this file**, 10:25Z, baseline `504`:

| config | `blob/v3.15.0/Objects/object.c#L273-L277` | `blob/v3.15.0/Objects/no_such_file.c#L1` |
|---|---|---|
| purba's settings | `504` | `504` |
| the same, with `remap` blob → raw | passes | `404`, and the output names the raw URL |

**purba's words for the online leg**, which a remap changes, quoted at `e2ccac8`:

| location | text |
|---|---|
| [`scripts/check-links.sh:2`](https://github.com/kalonji-tools/purba/blob/e2ccac8/scripts/check-links.sh?plain=1#L2) | "The links that do not reach their target as written" |
| [#523's spec](https://github.com/kalonji-tools/purba/issues/523) | "Each place that says what the link gate refuses says a link is refused when it does not reach its target as written. The gate itself does not change, so a `503` is still refused." |

## 3. Other projects

55 projects surveyed. 20 run a CI gate that fetches external links. Each count below is over those 20, at a pinned ref.

| practice | count | projects |
|---|---|---|
| accept a 5xx in config | 0 of 20 | — |
| exclude `/blob/` as such | 0 of 20 | — |
| fetch github.com on each pull request, and fail it | 9 of 20 | PyO3, polars, crates-io-auth-action, gpio-utils, async-book, opentelemetry.io, OTel semantic-conventions, OTel specification, jupyter_server |
| keep github.com off the blocking path | 11 of 20 | 5 on a schedule only; 3 exclude github.com; 2 offline or web links off on a pull request; 1 `continue-on-error` |
| remap `/blob/` links | 3 of 20 | both OTel repos send `/blob/…#frag` to raw; PyO3 drops the fragment |
| keep a cache between runs | 4 of 20 | PyO3, rustc-dev-guide, opentelemetry.io (a committed file), Homebrew |
| send a GitHub token | 13 of 20 | — |
| open an issue on failure | 6 of 20 | — |

- python/devguide excludes `r'https://github.com/.*'`, with the comment "Checks fail due to rate limits" [A].
- Sphinx marks a `503` "ignored", and still counts a `504` as broken ([`linkcheck.py` L615-L617](https://github.com/sphinx-doc/sphinx/blob/v9.2.0/sphinx/builders/linkcheck.py#L615-L617)) [A].
- During the outage, owenlamont/codestrand opened PR #90, "chore: accept GitHub gateway timeouts in link checks". It was not merged when read [A].

## 4. What GitHub documents

| source | date | what it says about a reader without a session | tier |
|---|---|---|---|
| [changelog](https://github.blog/changelog/2025-05-08-updated-rate-limits-for-unauthenticated-requests/) | 2025-05-08 | new limits for HTTPS clones, the REST API and raw.githubusercontent.com; no numbers, no status code, no web pages | [A] |
| [changelog](https://github.blog/changelog/2026-10-19-some-git-over-https-may-temporarily-require-authentication/) | 2026-10-19 | "During periods of unusual traffic, GitHub may temporarily require authentication for some clones and fetches of public repositories over HTTPS"; "Unrelated GitHub web and API traffic" is unaffected | [A] |
| docs.github.com | — | numbers only for the REST API: 60 an hour without a token; `403` or `429` past a limit, never `504` | [A] |
| [githubstatus.com](https://www.githubstatus.com/api/v2/incidents.json) | 2026-11-08 | "All Systems Operational" until 10:36Z, then "Increase in timeouts for some unauthenticated users", status investigating | [A] |
| other projects' reports | 2026-11-08 from about 08:40Z | at least 7 report the same `504`; 3 say `api.github.com/…/releases/assets/<id>` served the files while `github.com/…/releases/download/…` answered `504` | [B] |

## 5. The downloads

| tool | what it tries | when a release asset answers 5xx | other path | tier |
|---|---|---|---|---|
| `actions/setup-python` v7.1.0 | the runner's cache, then the manifest's `download_url` on github.com | 3 tries, 10 to 20 s apart; the token does not change the URL | `mirror` moves only the manifest | [A] |
| the runner image `ubuntu26/20261101.186` | — | — | caches Python 3.11 to 3.15, x64, **no free-threaded build**, and CodeQL **2.27.3** | [A] |
| `github/codeql-action` v4, default setup | the runner's cache for 2.27.4, then github.com | 1 streaming try, then 3 tries, 10 to 20 s apart | `tools:` cannot be set in default setup | [A] |
| `jdx/mise-action` v5.2.6 | its Actions cache | a pinned `version` comes only from github.com; 5 retries, 2 s apart | the CDN only when `version` is unset | [A] |
| mise 2026.9.14 | the lockfile's `url` | 3 retries | falls back to `api.github.com/…/assets/<id>` when the row holds `url_api`: 83 rows, none for `core:python` or `core:rust` | [A] |

purba's logs [A]: the `3.14t` job of run 42466709970 got three `504`s in 72 s. The CodeQL runs 42468231664 and 42468855507 found no 2.27.4 in the runner's cache, and got `504` on each try.

## What this research could not verify

- Whether this was an outage, or a lasting change for readers without a session. GitHub's docs say nothing either way [C].
- The CI installation token on a code page. The probes used a personal token [C].
- A signed-in `--cookie-jar` in CI, a ref with a slash, and a renamed repo's redirect on github.com itself [C].
- Whether any surveyed gate is a required check [C].
- Whether `api.github.com/…/releases/assets/<id>` also failed for purba's downloads [C].
