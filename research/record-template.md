# Which parts of the template make a record hard to read or write?

## What this found

The template says two different things about the chosen option. Its prose says Considered Options "holds what lost". Its form line puts the winner in that same list, as `{Chosen. Why it won.}`. Every one of the 95 records follows the form line. A reader looking for the decision finds it last in 71 records. In the other 24 records it can sit anywhere in the list. In 21 of those 24, the record answers several sub-questions inside one list. Only 6 of the 21 show where one sub-question ends and the next begins. The rules a command decides hold in all 95 records. The rules a reviewer decides are where writers trip. The human reviewer's top complaint, raised 35 times across 28 pull requests, is a Considered Options list that holds options nobody weighed. No written rule says "only the options this question weighed". Each of the three forms for the chosen option fixes a different part of this. Each one also costs something different. This file measures the three forms and does not pick one.

```
            what a reader meets today, top to bottom
 ┌────────────────────────────────────────────────────────────┐
 │ # proposition (the title)                                  │
 │ ## Context and Problem Statement                           │
 │ ## Considered Options                                      │
 │    - **option.** Rejected. ...                             │
 │    - **option.** Rejected. ...                             │
 │    - **option.** Chosen. ...   <- in 71 records: here, last│
 │      (in 21 records: 2 to 8 "Chosen." lines, mixed in)     │
 │ ## Decision Outcome                                        │
 │    **Reach:** ...                                          │
 │    The decision, in one sentence.  <- = title in 41        │
 │    detail ...                                              │
 │    **Downside:** - **cost.** ...                           │
 │ ## Confirmation                                            │
 └────────────────────────────────────────────────────────────┘
```

## Where the chosen option sits, across 95 records

```
last of the list          71  ███████████████████████████████████
several marked "Chosen."  21  ██████████
one, in the middle         2  █
the only option            1  ▏
```

| shape | records | options | chosen |
|---|---:|---:|---:|
| one chosen, last of two or more | 71 | 260 | 71 |
| several chosen | 21 | 172 | 63 |
| one chosen, in the middle | 2 | 12 | 2 |
| one option, which is chosen | 1 | 1 | 1 |
| all | 95 | 439 | 137 |

The 24 irregular records hold 179 options and 66 of them are chosen.

**Method.** An awk script reads each `- **...**` item under `## Considered Options` on `c6296bd`. An item is chosen when its text after the bold lead-in opens with `Chosen.`. The first word after each lead-in was tallied: `Chosen.` 137 times, a form of `Rejected` 297 times, and 5 other words. The words "chosen" appear 142 times in these lines. The other 5 are the word inside an option's own text, and each was read.

**A correction to the brief.** [a run exits 0 only when it checked something and every test passed](https://github.com/kalonji-tools/purba/blob/c6296bd/docs/decisions/a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md#considered-options) marks 5 of its 18 options chosen, not 6. The "first" record in the brief is [a change is specified before it is built](https://github.com/kalonji-tools/purba/blob/c6296bd/docs/decisions/a-change-is-specified-before-it-is-built.md#considered-options), which holds one option only.

### The 24 irregular records

| record | chosen of options | a mark between groups | title joins two predicates |
|---|---:|---|---|
| a line names its kind in the glossary's words | 8 of 13 | no | no |
| a run exits 0 only when it checked something and every test passed | 5 of 18 | no | yes |
| a panic inside a test gives that test an internal error | 5 of 10 | no | no |
| an interrupt stops the run, and a second one exits at once | 4 of 11 | no | yes |
| a test carries only purba's own decorators | 3 of 9 | no | no |
| only a changed input moves the order of design | 3 of 9 | no | yes |
| a test returns nothing, and the first part that sees a breach refuses it | 3 of 8 | no | yes |
| a failure's kind is what the part that first sees it observes | 3 of 7 | no | no |
| an interrupted run prints its count, and dies by the signal | 3 of 7 | no | yes |
| purba writes its scripts in bash until purba can test them | 3 of 7 | yes, prose | no |
| an implementation follows test-driven development | 3 of 6 | yes, prose | no |
| an artifact is rewritten until its direction is agreed | 2 of 11 | yes, prose | no |
| configuration takes the first location its tool reads | 2 of 8 | no | no |
| a blocked test has no outcome | 2 of 7 | no | no |
| one file marks every tracked path for each gate that can skip | 2 of 7 | yes, a question line | no |
| a commit outlives its review | 2 of 5 | yes, prose | no |
| a gate refuses each invariant a tool can see | 2 of 5 | no | no |
| each test runs in a process of its own | 2 of 5 | no | no |
| purba meets the next trait solver before it stabilizes | 2 of 5 | yes, a bold label | no |
| a case of each kind checks that no failure prints as another kind | 2 of 4 | no | no |
| an interrupt puts back no more than a normal end does | 2 of 4 | no | no |
| an error stops only the work that depends on it | 1 of 6, third | no | no |
| a test names two values of one type by two fixtures | 1 of 6, fifth | no | no |
| a change is specified before it is built | 1 of 1 | no | no |

Six of the 21 several-chosen records mark their groups. Each mark sits right after a chosen option. Two of the six use a heading-like line: a question in [one file marks every tracked path](https://github.com/kalonji-tools/purba/blob/c6296bd/docs/decisions/one-file-marks-every-tracked-path-for-each-gate-that-can-skip.md#considered-options), and a bold label in [purba meets the next trait solver](https://github.com/kalonji-tools/purba/blob/c6296bd/docs/decisions/purba-meets-the-next-trait-solver-before-it-stabilizes.md#considered-options). Fifteen records give no mark. Sixteen of the 21 have a title with one predicate, though the record chooses two to eight options.

The two "middle" records also hide a second question. In [an error stops only the work that depends on it](https://github.com/kalonji-tools/purba/blob/c6296bd/docs/decisions/an-error-stops-only-the-work-that-depends-on-it.md#considered-options), the three options after the chosen one reject other rules. Their answers sit as Decision Outcome bullets with no "Chosen." line.

## Each rule of the template, measured across 95 records

The rules come from [`.template.md`](https://github.com/kalonji-tools/purba/blob/c6296bd/docs/decisions/.template.md) on `c6296bd`, in the order the file states them. "Gate" means `scripts/check-records.sh` refuses a breach. I ran it from `c6296bd` on a copy of `docs/decisions/` at that commit, and it exited 0.

### Rules in the opening comment

| # | rule (template line) | who decides | follow | break or bend | method |
|---|---|---|---:|---|---|
| 1 | file is named for the proposition as a slug (2) | reviewer | 83 | 12 file names differ from their title's slug | slug of each `# ` title compared with the file name |
| 2 | delete this comment (3) | gate `comment` | 95 | 0 | gate run |
| 3 | the title is a proposition (5-7) | reviewer | 95 | 0 topic titles; 13 titles join two predicates with "and" | read all 95 titles |
| 4 | one sentence on one line (9-11) | gate `wrap` | 95 | 0 | gate run |
| 5 | one idea in one sentence (16) | post pass only | not measured | no command reads it | none ran |
| 6 | 25 words, 20 in a numbered list (17-19) | gate `long`, `step` | 95 | 0 | gate run |
| 7 | a procedure is a numbered list (18) | reviewer | not measured | 5 records hold a numbered list | grep `^[0-9]+\. ` |
| 8 | six sentences in a paragraph (20) | gate `para` | 95 | 0 | gate run |
| 9 | no -ing verb (21-22) | gate `ing` | 95 | 0 | gate run |
| 10 | simple tenses (23-24) | gate `tense` | 95 | 0 | gate run |
| 11 | active voice (25-27) | reported | not refused | 313 of 4,913 sentences passive | gate report |
| 12 | no em-dash (28-29) | gate `emdash` | 95 | 0 | gate run |
| 13 | bold opens a sentence (30-31) | gate `bold`, reviewer | 95 | 0 refused | gate run |
| 14 | one lead-in for each cost (32) | gate `leadins` | 95 | 0 | gate run |
| 15 | numbers in a table, not a sentence (33) | reviewer | not counted exactly | at least 9 records put a measured count in a sentence, on 13 lines | grep for "N of M", "%" and "N commits/files/..." outside code spans |
| 16 | a number only where it justified the decision (34-36) | reviewer | not measured | judgement | none ran |
| 17 | Downside required, label alone, costs a list, no count (38-40) | gates `listless`, `preamble`, `counted` | 95 | 0 | gate run |
| 18 | the minimum that conveys the point (42-45) | reviewer | not measured | judgement | none ran |
| 19 | session residue stays out; Considered Options holds what lost (47-51) | reviewer | 0 by the literal reading | all 95 put a chosen option in the list | see the conflict below |
| 20 | Decision Outcome opens with the reach (57-61) | gate `reach` | 95 | 20 patterns in 6 records match no tracked file (reported) | gate run, and reach patterns matched against `git ls-tree c6296bd` |
| 21 | Confirmation names its issues; the link goes when the issue closes (63-65) | reviewer | see below | 20 closed issues stay linked, from 11 records | each linked issue's state read with `gh issue view` |
| 22 | a record answers one question (67-69) | reviewer | 74 | 21 records choose 2 to 8 options | "Chosen." count |

### The section skeleton

| # | rule (template line) | follow | break or bend | method |
|---|---|---:|---|---|
| S1 | four headings, in order (74-96) | 95 | 0; no record carries Consequences or a sub-heading | every `#` heading tallied |
| S2 | Context: why, current state, not a story (76-77) | not measured | 47 records open Context with a `\| fact \| source \|` table, and 83 hold a table. No file prescribes the table. | `git grep 'fact \| source' c6296bd` outside the records |
| S3 | an option is `- **{option}.** {why}` (81) | 439 of 439 items | 297 of 302 lost options open with "Rejected", a word the template does not write | first word after each lead-in |
| S4 | the chosen option is marked `Chosen.` (82) | 137 marks, in 95 records | position varies, see above | as above |
| S5 | only list items in Considered Options (79-82) | 84 | 11 records put prose or a table in the list; 6 of these mark groups | lines that are not items or nested |
| S6 | `**Reach:**`, then the decision in one sentence (86-88) | 67 put it alone in a paragraph | 28 continue that paragraph; 75 bold the sentence, which the template does not ask | first paragraph after the reach |
| S7 | the Downside closes Decision Outcome (92-94) | 93 | 2 records put text after the costs | lines after the cost list |
| S8 | a cost is `- **{the cost}.** {What it costs, and to whom.}` (94) | 319 of 343 costs | 24 costs in 17 records stop at the bold lead-in | text after each cost's lead-in |
| S9 | Confirmation names the command or the setting (98-99) | 89 hold a code span; 60 name a `mise run` task | 6 hold no code span | grep |
| S10 | each issue titled by the question it asks (101) | 13 of 38 links | 25 link texts are not questions; 37 of 38 equal the issue's current title | link text compared with `gh issue view` |

### Three bends worth a picture

**The closed issues.** Rule 21 knows one use of an issue link: the intent, which goes when the issue closes. Of 30 issues linked from Confirmations, 20 are closed. All 21 lines that link them use the issue as evidence: "measured in", "recorded on", "decided this floor". The template names no evidence link, so it says these 11 records should have dropped them.

```
  rule 21 says           what 11 records do
  ┌────────────┐         ┌─────────────────────────────┐
  │ open issue │──closes─▶ link goes │ "measured in [closed issue]" │
  └────────────┘         └─────────────────────────────┘
```

**The filenames.** 12 titles moved after the file was named. For example, `significance-is-declared-not-detected.md` now carries "Architectural significance is declared, not detected". A record is rewritten in place, so its file name and its title drift apart.

**The unwritten conventions.** Writers keep three forms that no file prescribes: "Rejected." on a lost option (297 of 302), a bold decision sentence (75 of 95), and a `| fact | source |` table in Context (47 of 95).

### The conflict over the chosen option

```
 template line 49-51 (prose)          template line 81-82 (form)
 "Considered Options ... holds        - **{option}.** {Why it lost.}
  what lost rather than how the       - **{option}.** {Chosen. Why it won.}
  work arrived there."
          │                                     │
          └──────────── 95 of 95 records follow the form ──┘
```

The form line dates from the first template (`c42a0c1`, 2026-09-11). The "holds what lost" sentence entered on 2026-09-30 (`1bfb953`). One reading has the sentence contrast "what lost" with "how the work arrived there", which leaves room for the winner. The other reading has it exclude the winner. Read the second way, all 95 records and the template's own line 82 break it.

## Where writers and reviewers tripped

**Method.** `gh pr list --state merged --limit 500` returned 295 merged pull requests. 213 of them touch `docs/decisions/`, from #56 (2026-09-11) to #795 (2026-11-05). A helper agent read all 213 with 639 API calls: inline review comments, reviews, and issue comments. It kept 141 of 216 human inline comments as being about a record's text or form. It kept 505 of 974 post-pass items as being about a record, and gave them 517 rule assignments. It skipped the agent's 207 replies and the bot's 43 notices. Each kept item was traced to one rule by one reader, in one pass. I re-fetched 5 of the quoted comments, and each matched its author, its file and its words.

### Which rules trip most

| rule | human inline | human review | post pass | gated? |
|---|---:|---:|---:|---|
| Considered Options holds only what was weighed (rule 19) | 35 | 0 | 4 | no |
| minimum that conveys the point (rule 18) | 18 | 2 | 19 | no |
| one question per record (rule 22) | 5 | 0 | 15 | no |
| a frozen number (rule 16) | 5 | 0 | 7 | no |
| issues in Confirmation (rule 21) | 4 | 1 | 6 | no |
| Context is current state (S2) | 3 | 0 | 4 | no |
| the reach (rule 20) | 3 | 0 | 17 | partly |
| the Downside (rule 17, S8) | 2 | 0 | 11 | partly |
| one idea in one sentence (rule 5) | 1 | 0 | 48 | no |
| active voice (rule 11) | 0 | 0 | 25 | reported |
| Confirmation names the check (S9) | 0 | 0 | 20 | no |
| the option form (S3, S4) | 0 | 0 | 6 | no |
| the title (rule 3) | 0 | 0 | 5 | no |
| -ing, paragraph, line, tense, table, lead-in, length, issue number, citation | 0 | 0 | 21 together | yes |
| slug, em-dash, bold, sections, decision sentence | 0 | 0 | 0 | mostly yes |

Outside the template's rules: figure or table choice drew 10 human comments and 1 review. Title or filename length drew 4 comments and 1 review, and no rule speaks of it. STE rules that the prose record holds and the template does not drew 30 post-pass items. Claims that were false, unclear or badly linked drew 51 human comments and 279 post-pass items.

```
 human trips by rule (76 on template rules)
 weighed-only options  35 ███████████████████████████████████
 minimum               18 ██████████████████
 one question           5 █████
 frozen number          5 █████
 issues in Confirmation 4 ████
 other rules            9 █████████
```

The gated rules almost never trip a human. The rules that trip most are judgements with no gate.

### The weighed-only complaint, quoted

The human drew the line many times, and from #239 to #719.

- "you do not need to add alternatives if none were presented." ([#239](https://github.com/kalonji-tools/purba/pull/239#discussion_r4168425300)), on the record that now holds one option.
- "why are we padding considered options?" ([#516](https://github.com/kalonji-tools/purba/pull/516#discussion_r4247692391))
- "why are we consistently inflating the options. some of these options weren't even discussed" ([#546](https://github.com/kalonji-tools/purba/pull/546#discussion_r4262910927))
- "some of these options are subset/tangent of a option discussion. please reduce to the actual" ([#480](https://github.com/kalonji-tools/purba/pull/480#discussion_r4240709419))
- "I agree keep the options of the title's question" ([#524](https://github.com/kalonji-tools/purba/pull/524#discussion_r4252328704))
- "please keep it to the four I opted to go deeper into" ([#702](https://github.com/kalonji-tools/purba/pull/702#discussion_r4404504836))

The agent's replies name the cause: "Four of the eight were columns that I drew in the rounds" ([#682](https://github.com/kalonji-tools/purba/pull/682#discussion_r4386497198)). The template's skeleton shows a lost option above the chosen one. The #239 comment suggests a writer reads that shape as a demand for rivals. That is an inference, not a measure.

[An artifact holds the minimum that conveys its point](https://github.com/kalonji-tools/purba/blob/c6296bd/docs/decisions/an-artifact-holds-the-minimum-that-conveys-its-point.md#decision-outcome) keeps the categories a review has named. It names two: "what a session leaves behind" and a count on the Downside label. An option nobody weighed is not one of them, though it is the most frequent human trip.

### The one-question complaint, quoted

- "we are trying to resolve two decision in one record this should be split in two" ([#702](https://github.com/kalonji-tools/purba/pull/702#discussion_r4406050284)), on a record that still chooses 3 of 8 options.
- Post pass: "The marks record asks two questions in Considered Options ... not fixed: the reviewer decides" ([#511](https://github.com/kalonji-tools/purba/pull/511#issuecomment-6120618995)). That record still holds both questions.
- Post pass: "Considered Options left out the option that lost the second question." ([#665](https://github.com/kalonji-tools/purba/pull/665#issuecomment-6415558875))
- Post pass: ":29 says what the chosen option is, and not why it won." ([#520](https://github.com/kalonji-tools/purba/pull/520#issuecomment-6126786558))

### The earlier template defects

| defect | rule it touched | what it shows |
|---|---|---|
| [#249](https://github.com/kalonji-tools/purba/issues/249) | the placeholders | angle brackets rendered as nothing on GitHub |
| [#379](https://github.com/kalonji-tools/purba/issues/379) | the prose rules | a changed record does not link the template, so no link reaches the prose record |
| [#396](https://github.com/kalonji-tools/purba/issues/396) | rule 5 | the template claimed a borrowed rule the prose record did not hold |
| [#551](https://github.com/kalonji-tools/purba/issues/551) | the glob | a script counted the template as a record |

All four come from one cause. The template's rules live in an HTML comment that no command reads. The prose record's Downside states the same cost: "`.template.md` sits outside the glob, and its prose lives inside a comment the same command refuses elsewhere."

## Three forms for the chosen option

Each sketch uses [a blocked test has no outcome](https://github.com/kalonji-tools/purba/blob/c6296bd/docs/decisions/a-blocked-test-has-no-outcome.md#considered-options). It chooses options 4 and 7 of 7. It answers two sub-questions. Q1 asks what a test prints when an error outside it blocks it. Q2 asks what a test prints when its own error blocks it. Option 5 ("prints as `stopped`") belongs to Q1, yet it sits after Q1's chosen option.

### Today

```
## Considered Options
  1 Each test carries the error.                 Rejected.   Q1
  2 One error, its tests print nothing.          Rejected.   Q1
  3 Each blocked test has the outcome unknown.   Rejected.   Q1
  4 One error, each test prints as blocked.      Chosen.     Q1
  5 Each blocked test prints as `stopped`.       Rejected.   Q1
  6 Own error: prints blocked, names the error.  Rejected.   Q2
  7 Own error: prints that error, counts blocked. Chosen.    Q2
## Decision Outcome
  Reach ...
  A blocked test has no outcome.
```

### Form 1: Considered Options holds only what lost, and Decision Outcome holds the winner

```
## Considered Options
  - Each test carries the error.                 Rejected. ...
  - One error, its tests print nothing.          Rejected. ...
  - Each blocked test has the outcome unknown.   Rejected. ...
  - Each blocked test prints as `stopped`.       Rejected. ...
  - Own error: prints blocked, names the error.  Rejected. ...
## Decision Outcome
  Reach ...
  A blocked test has no outcome.
  Chosen: one error, each test prints as blocked.       why ...
  Chosen: own error prints that error, counts blocked.  why ...
```

| measure | value |
|---|---|
| records that change | all 95, because each holds a "Chosen." line; 137 items move |
| of the 24 irregular | 66 items move |
| records whose Considered Options goes empty | 1 (the one-option record); the gate accepts an empty section, probed |
| records whose outcome sentence already equals the title | 41, so a chosen line is a third statement of the decision |
| what moves | a reason of 14.9 words on average, against 20.1 for a lost one |
| precedent | MADR puts `Chosen option: "...", because ...` in Decision Outcome, after the list. Microsoft lists "Options considered" before "Decision outcome". |

**What is lost.** A lost option no longer sits beside the winner it lost to. In the 21 several-chosen records, a reader cannot tell which rejected option competed with which winner. Fifteen of them give no other mark. MADR still lists the chosen option among the options, so MADR is a precedent for the location, not for removal.

### Form 2: a heading for each sub-question, and the winner closes its group

```
## Considered Options
### What does a test print when an error outside it blocks it?
  - Each test carries the error.                 Rejected. ...
  - One error, its tests print nothing.          Rejected. ...
  - Each blocked test has the outcome unknown.   Rejected. ...
  - Each blocked test prints as `stopped`.       Rejected. ...
  - One error, each test prints as blocked.      Chosen. ...
### What does a test print when its own error blocks it?
  - Own error: prints blocked, names the error.  Rejected. ...
  - Own error: prints that error, counts blocked. Chosen. ...
## Decision Outcome
```

| measure | value |
|---|---|
| records that change | 23 (21 several-chosen and 2 middle) |
| records that stay as they are | 72 (71 regular and 1 one-option), if a one-question record carries no heading |
| records that already mark groups | 6, in prose; 2 of them with a question or a bold label |
| records with options after the last winner | 3 several-chosen, plus the 2 middle |
| records with two winners in a row | 5, so a group can hold a winner with no rival |
| gate | accepts a `###` heading inside Considered Options (probed on a copy: exit 0) |
| precedent | arc42 section 9 offers "separate sections per decision". No surveyed template puts sub-questions inside one record. |

**What is lost.** The winner stays inside the list, so the person still scans for it, though now within a short group. Each heading is a question in the record body. That makes plain that the record answers several questions, which line 67 of the template forbids. The position of a winner does not show where its group ends. In the sketch record, option 5 belongs to Q1 and sits after Q1's winner. So a writer must read each of the 15 unmarked records to split it.

### Form 3: one question for each record

```
 a-blocked-test-has-no-outcome.md            (today: 2 of 7 chosen)
        │ split
        ├── a test that an outside error blocks prints as blocked
        │     options 1 2 3 5 rejected, 4 chosen (last)
        └── a test whose own error blocks it prints that error
              option 6 rejected, 7 chosen (last)
        each links the other in one line
```

| measure | value |
|---|---|
| records that split | 21 |
| new records, at most | 42 (the 21 hold 63 winners), so 95 grows to at most 137 |
| the 2 middle records | each moves its winner last, or splits off its trailing options |
| records that stay as they are | 72 |
| after the split | the winner is last in every record, as in the 71 today |
| precedent | Nygard: "One ADR describes one significant decision". Microsoft: "Log each phase as its own decision record." Every surveyed template holds one decision. |

**What is lost.** A shared Context is written twice, or one record leans on the other. The Downside of the source record splits too. Some winners may be details of one decision and not separate questions: the 5 records with two winners in a row hint at that. The count of 42 is an upper bound for that reason. More records also add to the folder that [How can a person find a record by its kind?](https://github.com/kalonji-tools/purba/issues/799) asks about. [A new question gets a new record](https://github.com/kalonji-tools/purba/blob/c6296bd/docs/decisions/a-new-question-gets-a-new-record.md#decision-outcome) asks for a split only when an issue asks a new question. Its Confirmation asks that "each title states every decision its record holds". By that test, a several-chosen record breaks nothing when its title states each decision.

### The three forms side by side

| | Form 1: winner in Decision Outcome | Form 2: a heading per sub-question | Form 3: one question per record |
|---|---|---|---|
| where the winner sits | its own place, after the reach | last of its group | last of the list |
| records that change | 95 | 23 | 21 split, 2 reorder |
| new records | 0 | 0 | up to 42 |
| the loser beside its winner | no | yes | yes |
| the sub-questions shown | no | yes, as headings | yes, as titles |
| template line 49-51 ("holds what lost") | matches | conflicts as today | conflicts as today |
| template line 67 ("one question") | unchanged | conflicts openly | matches |

## Where other templates place the chosen option

A helper agent fetched each source. I re-read the MADR template, the Microsoft page, Nygard's post and Zimmermann's page from the saved downloads.

| template | order of the parts | where the winner sits | where its reasons sit | rejected options | winner also in the list? |
|---|---|---|---|---|---|
| [MADR 4.0.0](https://raw.githubusercontent.com/adr/madr/main/template/adr-template.md) | Context, Decision Drivers, Considered Options, Decision Outcome, Pros and Cons | Decision Outcome: `Chosen option: "{title of option 1}", because {justification}` | the "because" clause, and Pros and Cons | titles in Considered Options, then Pros and Cons | yes |
| [MADR minimal](https://raw.githubusercontent.com/adr/madr/main/template/adr-template-minimal.md) | Context, Considered Options, Decision Outcome, Consequences | Decision Outcome | the "because" clause | titles only | yes |
| [Nygard, 2011](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions) | Title, Context, Decision, Status, Consequences | Decision: "We will ..." | Context gives the forces | none; "alternative" appears 0 times | no list |
| [Y-statement, Zimmermann](https://www.ozimmer.ch/practices/2020/04/27/ArchitectureDecisionMaking.html) | context, facing, we decided for, and neglected, to achieve, accepting that | "we decided for", before the losers | "to achieve" and "accepting that", after both | "and neglected: alternatives not chosen (not to be forgotten!)" | no, a separate clause |
| [Tyree and Akerman, 2005](https://www.utdallas.edu/~chung/SA/zz-Impreso-architecture_decisions-tyree-05.pdf) | Issue, Decision, Status, ..., Positions, Argument, Implications | Decision, the second field, before the options | Argument, after Positions | Positions | not stated |
| [arc42 section 9](https://docs.arc42.org/section-9/) | suggests Nygard's structure | Decision | not placed | [tip 9-6](https://docs.arc42.org/tips/9-6/): "include rejected alternatives together with the reasons" | not stated |
| [Microsoft Well-Architected](https://learn.microsoft.com/en-us/azure/well-architected/architect-role/architecture-decision-record) | Problem statement, Options considered, Decision outcome, tradeoffs, confidence, status | Decision outcome, after the options | "include context and rationale", no place given | yes | not stated |
| [AWS Prescriptive Guidance](https://docs.aws.amazon.com/prescriptive-guidance/latest/architectural-decision-records/adr-process.html) | Title, Status, Date, Context, Decision, Consequences, Compliance, Notes | Decision | inside the Decision prose | no section | no list |
| [Henderson, business case](https://raw.githubusercontent.com/joelparkerhenderson/architecture-decision-record/main/locales/en-us/templates/decision-record-template-for-business-case/index.md) | criteria, candidates, analysis of each, Recommendation | Recommendation, last | the analysis of each candidate | yes | implied |

```
 where the winner sits, relative to the losers
 before the losers   Tyree/Akerman (Decision above Positions), Y-statement
 after the losers    MADR, Microsoft, Henderson business case
 no losers at all    Nygard, AWS (arc42 adds a tip, not a place)
 among the losers    MADR also names it in the list; purba puts it only there
```

No surveyed template states the winner only inside the list of options, as purba does. No surveyed template gives a form for one record that answers several sub-questions. Nygard writes "One ADR describes one significant decision for a specific project." Microsoft writes "Break one decision into multiple if an architectural decision is going to result in multiple phases." arc42 offers "separate sections per decision".

Two sources were not read. The [Medium post on Y-statements](https://medium.com/olzzio/y-statements-10eb07b5a177) returned 403 twice, so Zimmermann's own site stands in for it. The IEEE page of Tyree and Akerman (DOI 10.1109/MS.2005.27) is paywalled, so the scanned article stands in for it.

## Red team

| claim | what would make it wrong | what I checked |
|---|---|---|
| 71 records put the winner last | a winner marked by a word other than "Chosen." | the first word after every lead-in; only "Chosen." marks a winner, and the 5 other uses of "chosen" are option text |
| the gated rules hold in all 95 | a gate that did not run, or ran on another tree | ran `check-records.sh` from `c6296bd` on that commit's records; exit 0; the snapshot matched the extract (`diff -rq`) |
| a gated rule that holds everywhere still matters | a conformed corpus hides a dead rule | not tested; the gate's refusals on pull requests (#598, #761) show two rules fire |
| "holds what lost" conflicts with the form line | the sentence contrasts the losers with the process, not with the winner | both readings are stated; the conflict holds only under the second |
| winners close their groups in form 2 | a loser of one group sits after its winner | falsified: option 5 of the sketch record; the count of 23 is the records that change, not the effort |
| form 3 adds 42 records | some winners are details of one decision | stated as an upper bound; 5 records hold two winners in a row |
| the weighed-only complaint is the top human trip | the trips were classified by one reader, in one pass | 5 quotes re-fetched, each matched its author, file and words; ambiguous items were counted once, under their main rule |
| closed issues in Confirmation bend rule 21 | the links are intents left by mistake | read all 21 lines; each cites the issue as evidence |
| numbers sit in sentences in at least 9 records | the pattern caught version strings or exit codes | the 13 lines were read; spelled-out counts ("Sixteen of the eighteen") are missed, so 9 is a floor |
| 16 several-chosen records have a one-predicate title | the title still states each decision | not judged; the new-question record's own test is a reviewer's |
| no template puts the winner only in the list | a source not surveyed does | 9 templates read; a template outside them was not searched |

## Raw data

Per-record option counts (file, options, chosen, positions, shape):

```
a-blocked-test-has-no-outcome.md                                          7  2  4,7
a-case-of-each-kind-checks-that-no-failure-prints-as-another-kind.md      4  2  3,4
a-commit-outlives-its-review.md                                           5  2  3,5
a-failures-kind-is-what-the-part-that-first-sees-it-observes.md           7  3  2,4,6
a-gate-refuses-each-invariant-a-tool-can-see.md                           5  2  2,5
a-line-names-its-kind-in-the-glossarys-words.md                          13  8  2,4,6,7,8,10,11,13
an-artifact-is-rewritten-until-its-direction-is-agreed.md                11  2  6,11
an-error-stops-only-the-work-that-depends-on-it.md                        6  1  3   middle
an-implementation-follows-test-driven-development.md                      6  3  2,4,6
an-interrupted-run-prints-its-count-and-dies-by-the-signal.md             7  3  3,5,7
an-interrupt-puts-back-no-more-than-a-normal-end-does.md                  4  2  2,4
an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md             11  4  2,4,6,8
a-panic-inside-a-test-gives-that-test-an-internal-error.md               10  5  4,6,8,9,10
a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md    18  5  6,11,15,17,18
a-test-carries-only-purbas-own-decorators.md                              9  3  2,7,8
a-test-names-two-values-of-one-type-by-two-fixtures.md                    6  1  5   middle
a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md 8 3  4,6,8
configuration-takes-the-first-location-its-tool-reads.md                  8  2  6,8
each-test-runs-in-a-process-of-its-own.md                                 5  2  3,5
one-file-marks-every-tracked-path-for-each-gate-that-can-skip.md          7  2  4,7
only-a-changed-input-moves-the-order-of-design.md                         9  3  2,6,9
purba-meets-the-next-trait-solver-before-it-stabilizes.md                 5  2  2,5
purba-writes-its-scripts-in-bash-until-purba-can-test-them.md             7  3  3,5,7
a-change-is-specified-before-it-is-built.md                               1  1  1   one option
```

File names that differ from their title's slug:

```
a-commit-outlives-its-review.md                  A commit message outlives its review
a-gate-owns-the-mechanical.md                    A gate owns the mechanical standard
a-gate-refuses-each-invariant-a-tool-can-see.md  ..., from the first code it reads
a-loaded-module-joins-sys-modules-under-its-files-name.md  ..., without .py
a-record-is-rewritten-not-amended.md             A decision record is rewritten, not amended
a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md  ... a breach of a test ...
a-worker-loads-each-module-once.md               ..., and command hands each module once
execution-calls-a-test-by-its-functions-name.md  ..., and reporting prints the identity it is given
only-a-changed-input-moves-the-order-of-design.md  A script calculates the order of design, and ...
purba-parses-python-with-ruff.md                 purba parses Python with ruff's parser
purba-supports-a-python-with-two-years-left.md   purba supports a Python that has two years left
significance-is-declared-not-detected.md         Architectural significance is declared, not detected
```

Records whose Confirmation keeps a closed issue as evidence: a gate owns the mechanical standard, a record names every location where it applies, a standard a gate cannot decide does not become a gate, a task is named for what it does to the tree, a term belongs to the glossary, a workflow is granted an identity not a power, liability is recorded from the act that makes it true, one pull request answers one issue, purba records delegation and does not prevent it, purba supports a Python that has two years left, the person owns the Decision Outcome.

Pull requests read: all 213 merged pull requests that touch `docs/decisions/`, #56 to #795.
