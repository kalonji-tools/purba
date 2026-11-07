# Where the records assume an agent or a person where they mean a role

Issue: [Where do the records assume an agent or a person where they mean a role?](https://github.com/kalonji-tools/purba/issues/831)
Read at `7850ef1`, and the template also on `b07a65c`, the branch of the template's rules task.

## What was found

The records name a kind of entity where they mean a role in 77 places, in 33 files. Almost all of them come from six habits, and each habit is one sense of one word. "A person" most often means "any reader", "any author" or "by hand, not by a program". That contrast puts a person against a program and leaves out an agent, which is neither. "Agent" means the doer of a sentence in four lines, which is the line the person flagged. "Session" means the work that produced an artifact. "The person" stands for the one who decides, which is the architect.

Many more lines name a kind of entity on purpose. A sign-off, an acceptance, a credential, a delegation, a measured fact about agents, and the files written for an agent all rest on the kind. Those lines stay, and they are listed so that the person can check each call.

Five readers read every line: four read 23 to 26 records each, and one read the template, `CONTEXT.md`, `AGENTS.md`, `docs/agents/` and the comments of `scripts/check-records.sh`. Each `fix` row below was checked against the tree: its words are on the line it names, at `7850ef1`. Whether a line means a role is the readers' judgement, and the person decides each ✋.

## The six habits

| habit | the word means | lines |
|---|---|---:|
| 1. "agent" for the doer of a sentence | the doer | 4 |
| 2. "a person" for any reader, author or caller | a role | 31 |
| 3. "a person" for "by hand, not by a program" | an adverb, or a commit | 15 |
| 4. "the person" for the one who decides | ✋ the architect | 8 |
| 5. "session" for the work | the work | 5 |
| 6. "human" against "machine" for prose against parsed text | a reader, a parser | 4 |
| other: "maintainer", "developer", "user", "model", "an agent" for the integrator | a role, a machine | 10 |

The last row also holds "the person" for an account in `CONTEXT.md` Acceptance, which is ✋.

## The lines to fix

### 1. "agent" for the doer of a sentence

| file:line | the words | means |
|---|---|---|
| `docs/decisions/.template.md:26` (`b07a65c`: `:19`) | "the passive where the agent is unknown" | the doer |
| `purba-borrows-from-simplified-technical-english-rather-than-adopting-it.md:55` | "where the agent is unknown" | the doer |
| `purba-borrows-from-simplified-technical-english-rather-than-adopting-it.md:56` | "whether an agent is unknown" | the doer |
| `scripts/check-records.sh:272` | "where the agent is unknown" | the doer |

✋ Line 35 of the borrowed-rules record quotes each borrowed rule in the standard's own words. If the standard writes "agent", the fix marks the quote or rewords it with that cost named.

### 2. "a person" for any reader, author or caller

| file:line | the words | means |
|---|---|---|
| `a-blocked-test-has-no-outcome.md:14` | "proves that the person was told" | that the error was reported |
| `a-gate-owns-the-mechanical.md:6` | "Nobody can assume an agent read a convention" | any contributor |
| `a-pinned-candidate-decides-when-a-milestone-is-done.md:9`, `:23` | "a person who reads finished code" | a reviewer |
| `a-record-names-its-area-and-docs-readme-lists-the-records-by-area.md:5` | "A person looks for a record … an agent looks with `grep`" | a reader who browses, a reader who searches |
| `a-record-names-its-area-and-docs-readme-lists-the-records-by-area.md:17` | "A person cannot find a record" | a reader |
| `a-register-belongs-to-one-location.md:47` | "the person who writes" | the author |
| `a-test-carries-only-purbas-own-decorators.md:7`, `:16`, `:19`, `:20`, `:21`, `:32` | "A person who reads one test" | a reader of the test |
| `a-workflow-names-each-action-by-its-commit.md:42` | "a person adds it" | the author who adds it |
| `an-actor-is-what-it-does-not-what-it-is.md:46` | "before a person has to catch it" | before a reviewer has to |
| `an-actor-is-what-it-does-not-what-it-is.md:93` | "sentences a person wrote" | sentences an author wrote |
| `an-artifact-is-rewritten-until-its-direction-is-agreed.md:8` | "an agent read the body first" | a reader |
| `dependabot-proposes-each-newer-action.md:7` | "A person learns of an old action" | a reader of the run |
| `every-crate-lives-under-crates.md:74` | "A person who marks its directory" | a change that marks it |
| `liability-is-recorded-from-the-act-that-makes-it-true.md:42`, `:97` | "The agent is truthfully both / the author and the committer" | the author |
| `liability-is-recorded-from-the-act-that-makes-it-true.md:70` | "authorship \| the agent" | the author |
| `only-github-runs-what-lives-under-github.md:32`, `:33`, `:40`, `:80` | "a person reaches a task", "A script a person runs", "A person is a caller" | a caller outside GitHub |
| `purba-parses-python-with-ruff.md:88` | "reaches the person only when" | whoever raises the pin |
| `purba-supports-a-python-with-two-years-left.md:6` | "the next person who finds it" | the next reader |
| `purba-supports-a-python-with-two-years-left.md:40` | "the versions people run" | the versions in use |
| `the-zip-alerts-of-check-prose-are-dismissed-as-unused.md:46` | "the person who reads this record" | the reader |
| `CONTEXT.md:185` Reach | "goes to a person" | a reader |

### 3. "a person" for "by hand, not by a program"

| file:line | the words | means |
|---|---|---|
| `a-gate-owns-the-mechanical.md:119` | "a person fixed a real typo" | a fix by hand |
| `a-workflow-is-granted-an-identity-not-a-power.md:25`, `:76` | "a person to press a button", "waiting for a person" | an approval of the run |
| `a-workflow-is-granted-an-identity-not-a-power.md:39` | "not a person, bounds" | an account |
| `an-outside-contribution-is-applied-not-merged.md:21` | "grants a person rather than the Actions token" | an account with push access |
| `liability-is-recorded-from-the-act-that-makes-it-true.md:84` | "no person writes one" | nothing but the workflow |
| `purba-carries-no-tool-manager-beside-mise-and-no-compiler-of-its-own.md:97` | "one a person made" | a version linked outside mise |
| `purba-parses-python-with-ruff.md:59` | "A person raises the pin" | the pin is raised by hand |
| `purba-supports-a-python-with-two-years-left.md:65` | "A person who moves one" | a change that moves one |
| `the-nightly-is-named-by-a-date.md:38`, `:49` | "A person moves the date", "A person must move the date" | a commit moves the date |
| `the-zip-alerts-of-check-prose-are-dismissed-as-unused.md:18`, `:30`, `:39`, `:41` | "a person reopens", "A person dismisses" | by hand |

### 4. ✋ "the person" for the one who decides

| file:line | the words |
|---|---|
| `a-change-is-specified-before-it-is-built.md:45` | "the person who decides agrees to the spec" |
| `a-pinned-candidate-decides-when-a-milestone-is-done.md:34`, `:39`, `:53` | "The person reads one suite", "The person approves it", "back to the person" |
| `an-artifact-holds-the-minimum-that-conveys-its-point.md:56` | "The person raised it." |
| `CONTEXT.md:63` Design spec | "the person who decides agrees to the spec" |
| `CONTEXT.md:67` Desired-state suite | "The person approves it before the build starts." |
| `AGENTS.md:10` | "the person who decides agrees to the spec" |

The role is the architect, unless the rule means that a person must agree. No mechanism requires a person here, as it does for a sign-off. The person decides this sense once, for every line.

### 5. "session" for the work

| file:line | the words | means |
|---|---|---|
| `docs/decisions/.template.md:47` (`b07a65c`: `:32`) | "the session that wrote this" | the work that produced this |
| `a-term-belongs-to-the-glossary.md:18` | "A session answering a single question" | a reader |
| `an-artifact-holds-the-minimum-that-conveys-its-point.md:41`, `:48` | "a session leaves behind" | the work leaves behind |
| `an-artifact-is-rewritten-until-its-direction-is-agreed.md:36` | "the session that wrote it" | the work that wrote it |

### 6. "human" against "machine"

| file:line | the words | means |
|---|---|---|
| `a-commit-outlives-its-review.md:30` | "specify the human one" | the prose contract |
| `a-normalisers-style-is-chosen-for-the-human-who-reads-the-code.md:1`, `:18` | "for the human who reads the code, because a machine reads any style" | the reader of the code; a parser |
| `a-gate-owns-the-mechanical.md:36` | link text "for the human who reads the code" | follows that title |

The title changes, and the file keeps its name, as a decision record is rewritten, not amended now says.

### Other words

| file:line | the words | means |
|---|---|---|
| `a-workflow-is-granted-an-identity-not-a-power.md:26` | "purba has one maintainer" | one code owner |
| `a-workflow-is-granted-an-identity-not-a-power.md:63` | "A later maintainer" | anyone with admin access |
| `a-test-names-two-values-of-one-type-by-two-fixtures.md:18` | "each is a model" | a simulation |
| `purba-carries-no-tool-manager-beside-mise-and-no-compiler-of-its-own.md:28` | "the developer already has" | the machine's own |
| `the-nightly-is-named-by-a-date.md:55` | "a developer machine" | a local machine |
| `purba-ranks-an-honest-outcome-first.md:55`, `:62` | "An agent reads the result", "The result for an agent" | the integrator |
| `purba-writes-its-scripts-in-bash-until-purba-can-test-them.md:50`, `:54` | "first user", "share no user" | first suite to run; no reader |
| `CONTEXT.md:11` Acceptance ✋ | "the approval of a person whose account has write access" | an approving account with write access |

✋ Acceptance: `.github/scripts/standing-approver.sh` reads an account's permission, and the agent account holds `write`. The same words are in `.github/scripts/write-acceptance-trailer.sh:2`, `a-workflow-is-granted-an-identity-not-a-power.md:16` and `liability-is-recorded-from-the-act-that-makes-it-true.md:71`. The liability record names this case: nothing separates a person from an agent that holds the person's credentials. Rewording it to "an account" states the mechanism, and keeping "a person" states the intent.

## The lines that stay

The kind of entity is the point of each line below.

| why it stays | where |
|---|---|
| only a person signs, accepts or holds the origin right | liability `:5-6`, `:50`, `:71`, `:75`, `:80`, `:94`, `:118`, `:131`; a-workflow-is-granted `:5`, `:16`, `:26` ("the person who opens it"); a-change-is-specified `:92`; a-commit-outlives `:58`; only-github `:6`; dependabot `:41`; the-nightly `:49` ("for a person to sign"); `CONTEXT.md` Origin |
| a person answers for the Decision Outcome | the-person-owns-the-decision-outcome `:1`, `:13-15`, `:19-20`, `:28`, `:30`, `:34`, `:38`; significance `:5`, `:15`, `:33`, `:44`, `:48`, `:79-80` |
| delegation and credential custody are the subject | purba-records-delegation `:5`, `:7-8`, `:10`, `:14-15`, `:27`, `:39-41`, `:47`, `:50`; liability `:119-196`; a-change-is-specified `:46`, `:106`; a-new-question `:28` |
| the rule binds an agent | a-change-is-specified `:78`, `:80-81`, `:86`, `:109`; a-record-is-rewritten `:81`; purba-borrows `:79`; the-agent-that-marks-a-pull-request-ready-watches-it `:1`, `:8`, `:12`, `:20`, `:25`, `:30`, `:32` |
| the trailer names AI help | a-commit-outlives `:109`, `:113-115`, `:117-118`, `:124`, `:137` |
| a file written for an agent | a-change-is-specified `:125`; a-new-question `:68`; a-location-inherits `:48`; an-implementation-follows-tdd `:83`; an-artifact-is-rewritten `:171`; the-glossary-stays-open `:44`, `:60`; `AGENTS.md` lines 1, 41, 50, 60, 68, 82, 87; `docs/agents/post-pass.md`; `CONTEXT.md` Handler |
| a program, by its kind | a-change-is-specified `:51`; purba-carries-no-tool-manager `:83`; an-actor `:53` |
| a measured fact about agents or people | a-test-names-the-fixture-of-each-input `:11`; a-test-names-two-values `:16`; a-suppression `:18-19`; an-actor `:15`, `:19`, `:21`; purba-ranks-an-honest-outcome-first `:11`, `:22`; a-standard-a-gate-cannot-decide `:9`, `:13-14` |
| the actor record's own subject | an-actor `:25-26`, `:47`, `:55`, `:64`, `:82`, `:85` |
| a quote, or a word named as a word | a-suppression `:12`; a-term `:30`; an-interrupted-run `:15`; liability `:104`, `:112` (history) |
| names both kinds, to say either fills the role | a-suppression `:40`; purba-runs-inside-the-testers-interpreter `:37`; a-gate-owns-the-mechanical `:5` |
| GitHub's name for a permission | `maintainer` in an-outside-contribution `:25-26`, `:40`, `:56`, `:61-62`, `:64`, `:73`, liability `:117`, `:219`; `CONTEXT.md` Maintainer |

Two calls the readers asked the person to check:

- **significance `:5` "Human attention is the scarce resource".** It stays, because the scarce attention is that of the person who must approve.
- **the post pass is owed only on work an agent wrote** (a-change-is-specified `:78`). Every line in "the rule binds an agent" follows from it. Whether that rule should depend on the kind of entity is a question for that record, and this scan does not answer it.

## The files with no line

54 of the 97 records hold no line to fix or keep. `CONTEXT.md` holds 59 entries with none.

## How it was read

Each reader read every line of its files with `git show 7850ef1:<path> | cat -n`, judged each hit against the record's Decision Outcome, and checked its reading with a word list: agent, AI, LLM, model, Claude, bot, human, person, people, developer, user, maintainer, session, prompt, context, token. Each `fix` row was then checked by a script: its quoted words are on the line it names.

Assisted-by: Claude:claude-opus-5-5
