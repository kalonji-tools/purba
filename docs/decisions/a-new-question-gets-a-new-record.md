# A new question gets a new record

## Context and Problem Statement

The [architect](../../CONTEXT.md#architect) chooses purba's direction, so it needs every decision at once.
Every other reader arrives at one record through a link.
No file sends a reader through the whole set.

[A decision record is rewritten, not amended](a-record-is-rewritten-not-amended.md) makes `ls` and `grep` the whole interface, and each title is a proposition.
So `ls` shows a decision only while a title states it.

The records on `main` at `6b79589`, read at 238 words a minute:

| | words | minutes |
|---|---:|---:|
| the titles | 180 | under 1 |
| the records as merged | 22,403 | 94 |
| what rewrites added after the merge | 5,753 | 24 |

The rewrites went mostly into a few records, and the largest absorbed the answers of later issues.

| `liability-is-recorded-from-the-act-that-makes-it-true` at `6b79589` | |
|---|---:|
| words at merge | 1,556 |
| words | 3,309 |
| distinct issues that changed it | 18 |

Its Confirmation answered a question a later issue asked: whether a person's act can be made impossible for their agent to forge.
The title did not state that second decision, so `ls` did not show it.

## Considered Options

- **A budget for the length of the set.** Rejected. The architect reads the set through its titles, so the length of the bodies is not its cost. A ceiling also refuses a new decision because older records are long.
- **A target length for each record.** Rejected. `.template.md` set three minutes from the first record, and two records in twenty-four met it.
- **Extend the record whose subject is closest.** Rejected. That was the practice, and the record above is its result.
- **A new question gets a new record.** Chosen. The test starts from the question an issue asks, which a reviewer can read, and it keeps each decision in a title.

## Decision Outcome

**A new question gets a new record.**
When an issue asks a question that no title on `main` answers, its answer goes in a new record.
The new record and the older one link each other, each in one line.
A reviewer decides whether a title answers a question, because no command can.

**The architect reads the set through its titles, and reads a body where two records meet.**

**Nothing budgets the length of the set or of a record.**
A record holds the minimum that conveys its point, and a reviewer judges that.

**Downside:**

- **Nothing limits the length of the set or of a record.** The architect's view stays short only while each title states every decision its record holds.
- **The number of records grows faster.** Each split also costs a record rewrite and a code owner review.
- **Whether a title answers a question is a judgement.** Two reviewers can read one title differently, and this record calls neither wrong.

## Confirmation

| what proves it | where |
|---|---|
| each title states every decision its record holds | the reviewer of each change to `docs/decisions/` |
| the answer to a new question went to a new record | the same reviewer, read against the issue the change answers |

[`AGENTS.md`](../../AGENTS.md) carries the rule to an agent before it writes a record, and `.template.md` carries it to a writer who starts one.

Each second decision a reviewer found in a record when this was decided has a record of its own.
