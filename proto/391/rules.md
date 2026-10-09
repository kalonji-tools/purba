# The prose rules the sweep takes, one at a time

Each rule below is quoted from the Decision Outcome of
`docs/decisions/purba-borrows-from-simplified-technical-english-rather-than-adopting-it.md`
at the base of the pull request, except the control.
The record's reach is `docs/decisions/*.md`, so the sweep reads only lines added there.

The four rules that the record says a command refuses (25 words, six sentences,
the `-ing` form, the simple tenses) are not swept: `mise run records` owns them.

## R1. The active voice

The record: "the active voice | reports".
"The rule admits the passive where the agent is unknown."

A line breaks R1 when a clause is in the passive voice and the text names, or the
reader can name, who or what does the act.

## R2. No wordy or formal word

The record: "no wordy or formal word | reports".

A line breaks R2 when it uses a long or formal word where a short common word
says the same thing.

## R3. No part of a sentence left out

The record: "no part of a sentence left out | reports".

A line breaks R3 when a sentence drops a word it needs to be a full sentence,
such as an article, a verb or a subject. A bold lead-in followed by a full
sentence is a heading, not a fragment. A table row is not a sentence.

## C1. One idea in one sentence (control: no record states it)

`.template.md` says: "One idea in one sentence."
The STE record quotes this in its Context only, and its table of borrowed rules
does not carry it. Under "the rules are the records only", the audit does not
own C1. The sweep runs it to measure what a record for it would buy.

A line breaks C1 when one sentence carries two ideas that could each stand as a
sentence, such as two independent clauses joined with `, and`.
