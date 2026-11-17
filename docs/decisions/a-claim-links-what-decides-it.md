# A claim links what decides it

## Context and Problem Statement

An agent states facts in an issue, a design spec, a pull request body, a post pass and a record.
The person decides on them, and cannot check a fact that carries nothing.
A report of the agent's sessions found that the person who decides corrected the same gaps by hand.

| fact | source |
|---|---|
| the report covers the sessions from 2026-11-02 to 2026-11-17 | unmeasured: the report is not published. [The Notes of the map of this rule](https://github.com/kalonji-tools/purba/issues/1052) state each fact of the report |
| a claim was posted with no probe behind it, and a count later showed it false | unmeasured, as above |
| a behaviour of purba was probed on a script that imitates it, not on a wheel built from `main` | unmeasured, as above |
| a concept that purba borrows was defined in the agent's own words, not in its source's | unmeasured, as above |
| `AGENTS.md` asks for the primary source of a concept only in a comment in the tree | [`AGENTS.md` L71](https://github.com/kalonji-tools/purba/blob/ee8ac2725ae76f8ca233c4e926be03332b577770/AGENTS.md?plain=1#L71) |
| 58 of 118 records hold a `\| fact \| source \|` table | [the probe of the claim issue](https://github.com/kalonji-tools/purba/issues/1053#issuecomment-6953449447) |
| `.template.md`, `AGENTS.md`, `CONTEXT.md`, `docs/agents`, `scripts` and `.config` do not write that header | the same |
| step 3 of the post pass reads the diff, the commit messages and the records, and not the pull request body or the design spec | [`post-pass.md` L9](https://github.com/kalonji-tools/purba/blob/3bbcbc0fa0ce89a6d400fcc9822b4581c2d6b408/docs/agents/post-pass.md?plain=1#L9) |

## Considered Options

- **A claim carries nothing, as on `main`.** The person cannot check a count, and a false one passes into the decision.
- **A script that imitates purba stands in for it in a probe.** It does not have the layout that purba ships.
- **Only a comment in the tree cites the primary source of a concept, as [`AGENTS.md` L71](https://github.com/kalonji-tools/purba/blob/ee8ac2725ae76f8ca233c4e926be03332b577770/AGENTS.md?plain=1#L71) asks.** A record that borrows a concept escapes the rule. Its meaning drifts from the source.
- **The post pass reads only the diff, the commit messages and the records, as on `main`.** The claims of the pull request body and the design spec stay unread.
- **The reach names paths alone, so `mise run list:records` prints this record only where a path matches.** [A record names every location where it applies](a-record-names-every-location-where-it-applies.md#decision-outcome) reaches a location with no path by its word.
- **A change adds the table to each record without one now.** The sources of old frozen facts are hard to find. A sweep invites guessed links.
- **The records that use Walking skeleton and Stand-in, the terms of Cockburn and Meszaros, get a source row now.** [The table issue found](https://github.com/kalonji-tools/purba/issues/1065#issuecomment-6956259715) that no record adopts either concept.

## Decision Outcome

**Reach:** [issue](../../CONTEXT.md#issue) [design spec](../../CONTEXT.md#design-spec) [pull request](../../CONTEXT.md#pull-request) [post pass](../../CONTEXT.md#post-pass) `docs/decisions/*.md` `/AGENTS.md` `/CONTEXT.md` `docs/decisions/.template.md` `docs/agents/post-pass.md`

**Area:** writing

**Each [claim](../../CONTEXT.md#claim) that an agent posts links what decides it.**
The rule holds in an issue body, a design spec, a pull request body, a post pass and a record.
The person decides on each of them.

| the claim states | it links |
|---|---|
| a line of a file | the line, at a commit |
| what a command shows: a count, "no test does X", a behaviour | a [probe](../../CONTEXT.md#probe), whose command shows its matcher |

- **A commit message is not in the rule.** [A commit message outlives its review](a-commit-outlives-its-review.md#decision-outcome) keeps its body to what must outlive the review.
- **A probe of purba runs on a wheel built from a named commit.** A script that imitates purba never stands in for it.
- **A probe of another tool names the version of that tool.**
- **A claim that no probe can decide says that it is unmeasured.** It says why.
- **A judgement says that it is one, and links nothing.**
- **A claim links only what decides it.** [Evidence density](an-artifact-holds-the-minimum-that-conveys-its-point.md#decision-outcome) counts any link.
- **The report of a [blind rating](../../CONTEXT.md#blind-rating) holds claims.** [A form that a tester meets is rated blind before it is decided](a-form-that-a-tester-meets-is-rated-blind-before-it-is-decided.md#decision-outcome) says what those claims link.

**A borrowed concept:**

- The record that adopts a concept that purba borrows cites the originator's own text, in a row of its `| fact | source |` table.
- Where the writer finds no such text, the row cites another source and labels it `secondary`.
- That record says that purba departs from the source. It says why.
- `CONTEXT.md` holds the meaning and links nothing, as [a term belongs to the glossary](a-term-belongs-to-the-glossary.md#decision-outcome) asks.
- The writer cites a concept that no record adopts in the design spec of the change that adds or rewrites its term.

**The post pass:**

- A context that did not write the change reads the pull request body and the design spec. It checks three things for each claim:
  1. The claim links a line at a commit or a probe, or says that it is unmeasured or a judgement.
  2. A probe names the head that it ran on.
  3. The command of a probe counts what the sentence says.
- The post pass runs no probe again. A count taken at one head is true at that head.
- The author rewrites a breach in the pull request body, which freezes only at the merge.
- The type of a design spec freezes it. The author marks a breach in a frozen spec with a pointer to the post pass, and writes a line under "What you must decide". [An artifact is rewritten until its direction is agreed](an-artifact-is-rewritten-until-its-direction-is-agreed.md#decision-outcome) lets a frozen artifact gain a pointer, and never a changed line.
- The reach names glossary words, so `mise run list:records` prints this record on every pull request. [`records.sh` L160-L175](https://github.com/kalonji-tools/purba/blob/3bbcbc0fa0ce89a6d400fcc9822b4581c2d6b408/scripts/records.sh?plain=1#L160-L175) decides it.

**The fact and source table:**

- The Context of a record puts each fact in a `| fact | source |` table.
- In a record written before this rule, the writer adds a row for each line of a change's diff that states a fact.
- A reworded fact owes a row too.
- `Area:` and the bold sentence of a Decision Outcome are not facts.
- Where a fact lost its source, the writer finds its commit with `git log -S`. The writer then reads the pull request and the issue of that commit. Where none of them holds the source, the row says `unmeasured`.
- The number of such a fact stays frozen. The writer never probes it again to replace it.

**Downside:**

- **No command decides whether a sentence is a claim or a judgement.** A writer can label a claim a judgement, and only an entity that reads the sentence sees it.
- **A record written before this rule keeps facts with no row** until a change rewrites them.
- **Step 5 of the post pass costs one more context on each pull request.**

## Confirmation

No command checks a claim.
Step 5 of [the post pass](../agents/post-pass.md) judges each claim in the pull request body and the design spec.
The reviewer judges the claims of the issue and the records.

[What does a claim in a spec or a resolution carry, so that its reader can check it?](https://github.com/kalonji-tools/purba/issues/1053) decided the rule.
[Where does a writer cite the primary source of a concept that purba borrows?](https://github.com/kalonji-tools/purba/issues/1054) decided the borrowed concept.
[Does the post pass judge each claim that a change posts in its pull request body and its spec?](https://github.com/kalonji-tools/purba/issues/1064) decided the post pass.
[Does each record get its fact and source table and the source of its borrowed concepts now, or when it is next rewritten?](https://github.com/kalonji-tools/purba/issues/1065) decided the table of an old record.
