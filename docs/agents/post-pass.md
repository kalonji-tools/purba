# Run a post pass

[A change is specified before it is built](../decisions/a-change-is-specified-before-it-is-built.md#decision-outcome) says when an agent owes a [post pass](../../CONTEXT.md#post-pass).

## The steps

1. Read the branch for what can be removed or improved.
2. Run `mise run list:records`. It prints the records on every pull request, the records this change names, and the record lines that may hold a stale sentence.
3. Judge the change against the records. Read the diff, the commit messages, and the Decision Outcome of each record on the first two lists. Read in full each record that the change edits. Judge each line on the third list. [A decision record is rewritten, not amended](../decisions/a-record-is-rewritten-not-amended.md#confirmation) says which cited issue makes a line stale.
4. Judge the prose. Run `mise run list:records -- --added 'docs/decisions/*.md'`, and judge each line it prints against every rule that [the prose record](../decisions/purba-borrows-from-simplified-technical-english-rather-than-adopting-it.md#decision-outcome) reports. The form `- **{option}.** {Chosen. Why it won.}` in `docs/decisions/.template.md` is not a breach.

Run steps 3 and 4 each in a context that did not write the change.
Run the two in parallel where you can.

## The comment

Post the pass as one comment on the pull request.
It carries these parts:

- the head that the pass read;
- the first two lists, as `mise run list:records` prints them;
- the number of lines on the third list;
- what can be removed or improved;
- each breach, with the commit that fixed it.

A stale sentence is a breach of [a decision record is rewritten, not amended](../decisions/a-record-is-rewritten-not-amended.md#decision-outcome).
The comment does not repeat the third list, because the command prints it again from the head.

Post the comment even when the pass finds no breach.
Also write each breach that you do not fix in the pull request body, under "What you must decide".

A new pass reads the whole change, and its comment links the pass it replaces.
Never edit an earlier pass.

## What belongs in this file

This section is the register of this file, and [a register belongs to one location](../decisions/a-register-belongs-to-one-location.md#confirmation) names it.

A line belongs here when it says how to run a post pass, or what its comment carries.
This file links a rule that a record holds, and never states it.
