# An artifact is rewritten until its direction is agreed

## Context and Problem Statement

purba re-scopes its own tickets, and nothing marks the part that stopped being true.
A re-scope arrives as a comment, the body it contradicts stays unmarked, and the reader reconciles the two wrongly.

On [Post the mandatory review threads that must be resolved before a merge](https://github.com/kalonji-tools/purba/issues/53), an agent read the body first and built design questions on a premise the re-scope had already killed.
That comment was present, correct, and had been read.
It lost to the body, because the body reads as current and carries no mark saying it is not.

The tracker had already answered half the question without deciding it, and the two halves disagree.

| part of an issue | what was done | n |
|---|---|---|
| title | rewritten, and the old one discarded from view | 11 |
| body | annotated in place, never rewritten | 6 |
| superseded comment | untouched | 6 |

Three premises the question rested on are false, and each was measured against the API.

| the premise | what the API does |
|---|---|
| a rewrite destroys the superseded text | `userContentEdits` returns a full snapshot at every version, oldest node first |
| a title cannot carry a mark, so it is the worst preserved part | `RenamedTitleEvent` records the previous title of all 11 renames, making it the best preserved part |
| a merged pull request description must never be edited, or the record of intent is lost | pull request bodies carry the same snapshots |

That history reaches GraphQL and the web interface and nothing else.
`gh issue view` returns the current body with no sign that another version exists, and that is what the misled reader was reading.

No version has an address.
The interface serves history from a popover, three candidate paths answer 404, and the edit node carries an identifier and no URL.
**A mark can point at what replaced a body and never at the body it replaced.**

A second behaviour was already running, unwritten.
No comment here has ever been edited outside the session that wrote it: the latest of 17 edits across 202 comments lands 86 minutes after posting, and both design spec edits are typo fixes.

## Considered Options

- **Strike the superseded spans in place.** Rejected. Tried on a live ticket: a strike across a multi sentence list item carrying bold and links renders as noise. It works on a heading and on nothing longer.
- **A banner at the top of the body.** Rejected. It marks the body and leaves the dead text persuasive, which is the failure rather than a fix for it.
- **A `superseded` label.** Rejected. It marks the issue and never the span, and the measured defect is span level.
- **Rewrite, and preserve the old body in a comment.** Rejected. The platform already keeps every version, so the copy is a second thing to maintain and to disagree with the first.
- **Rewrite, and link the prior version from the mark.** Rejected by measurement. No stored version is addressable, so the link cannot be written.
- **Rewrite, and carry one line pointing at what replaced it.** Chosen. The body then reads as current state, and pointing forward tells the reader why it changed where a backward link would only say that it changed.

A second question decides when rewriting stops being allowed.

- **Append only, never edit.** Rejected. It refuses a typo fix made four minutes after posting, and two of the eight design specs are exactly that.
- **Freeze once somebody has read it.** Rejected, because nothing reports reading. `viewerViewedState` is the caller's own private checkbox and returns `UNVIEWED` for files the caller wrote.
- **Freeze on a planning field.** Rejected. Four are defined here, and a value is invisible to anyone not signed in: one issue carries `Priority: Medium`, which reads as nothing to the public. A rule an outside contributor cannot evaluate is not a rule.
- **Freeze on something that happens to the artifact, such as a reply or a close.** Rejected. It makes the freeze point a judgement about what somebody did, so the record has to define acting, and the best available definition was relying rather than noticing, which is not a test anyone can run.
- **Freeze when the parties declare the direction agreed, by setting the issue type.** Chosen. It is a declaration rather than a detection, which is the move [architectural significance is declared, not detected](significance-is-declared-not-detected.md) already made here for the same reason: the property becomes true by definition and nothing has to judge it.

## Decision Outcome

An artifact is rewritten in place until its direction is agreed, and is frozen after.

**Agreement is declared, and the issue type is the declaration.**
Resolving a design spec ends by setting the type, which records that the parties agree on the direction.
`Task`, `Bug` and `Feature` therefore mean more than a taxonomy: each one says this was agreed.

Priority, effort and the date fields freeze nothing, and neither does assigning the issue.
They record how the work is scheduled, never that its direction is agreed.
This organisation defines four such fields beside the type, so the distinction is worth stating rather than inferring.

| artifact | frozen by |
|---|---|
| issue body, title and design spec | the type being set |
| pull request body | the merge |
| a comment recording a dated event | posting it |

While the type is `null` the issue is a draft, and its body and its spec are rewritten freely with no mark owed.
The declaration is public, it is a string comparison, and lifting it is recorded as `issue_type_removed`.

**Marking is not rewriting.**
A frozen artifact gains a pointer and never loses or changes content.
A mark is a signal and never a licence: what permits a rewrite is that the artifact is not frozen yet.

**A re-scope narrows, and anything else is two tickets.**

| the original question | what to do |
|---|---|
| survives with a tighter boundary | rename the title and rewrite the body, carrying one line naming the change |
| was answered, abandoned or replaced | close the original as `not_planned` carrying its answer, and open a new issue linking back |

The reason is duplicate work rather than tidiness: a re-purposed ticket hides the answer to the question it used to ask, so the next contributor asks it again and nobody can tell them it was already ruled out.
`state_reason` is public in the plain issue payload, so delivered and ruled out stay distinguishable.

**Two exceptions survive the freeze, both narrow.**
A typed issue may still have a typo or a wrong filename corrected, because the question never changed, and it may still be narrowed.
Anything that changes which question the number asks is two tickets.

**A closed issue is never re-scoped**, which follows: a re-purpose is a close and an open.

**The pull request freezes at merge rather than at its first reader.**
Every edit before merge is the author answering the reviewer, which is one conversation and not a second signed version.
Divergence found afterwards goes in a post merge debrief.

**A decision record is the exception to the freeze itself**, and [a record is rewritten, not amended](a-record-is-rewritten-not-amended.md) holds it.
A record that reads as history is a record nobody can trust on one reading.

**Superseding is a procedure, in this order.**

1. Post the replacement. It stands alone even when only part changed, and it carries what the superseded text held and why that was wrong.
2. Copy its URL.
3. Mark the superseded artifact with that URL, naming the span that died when only a span died.
4. Minimize a wholly superseded comment as `outdated`. One superseded in part keeps its live text visible and carries the pointer alone.

The order never collapses, because step 3 needs a URL that step 1 creates.
It is steps rather than a sentence because the sentence failed on its first live use, when its own author guessed a URL before the comment existed.

Step 1 is what makes step 4 safe.
A rewritten body carries its dead reasoning in a Considered and rejected section, and step 1 is the same obligation for a comment, which cannot be rewritten.
That makes the hidden comment redundant by construction rather than by assumption.

**A reinstatement is a new artifact, never an un-marking.**
Post the design again and say why the one that replaced it failed, in the same comment.
Every pointer then stays true and the history says the project went round the loop.

**Downside:** four costs, and the first lands on the day this merges.

- **Nothing in the tracker is frozen yet.** 79 of 80 issues carry no type, so every one of them becomes a draft. That is arguably correct, since none was ever agreed this way, and it means the rule protects nothing until types are set.
- **The freeze can be lifted.** Removing a type un-freezes an issue. The removal is public and recorded, so it cannot be done quietly, but nothing refuses it.
- **Two tickets cost more than one rename**, against a standing preference here for re-scoping over filing. Re-scoping is cheaper to write and more expensive to read, and this project has one writer and expects many readers.
- **Nothing enforces any of it.** Every check below reports and none gates, because an issue has no merge event.

## Confirmation

Three properties are decidable, and all are checked by hand.

| check | reads |
|---|---|
| an issue with a type has a design spec that was resolved | `issueType` and the comment list |
| a renamed title on a typed issue implies the body carries a re-scope line | `RenamedTitleEvent` and the body |
| a comment carrying a supersession prefix is minimized as `outdated`, and one minimized as `outdated` carries a prefix naming its replacement | `minimizedReason` and the body |

The last cross checks itself, and either half failing without the other names the step that was skipped.

Neither the first nor the second gates: an issue has no merge event, so a check could only label after the fact, and [a register belongs to one location](a-register-belongs-to-one-location.md) rejected a labelling workflow for that reason.

Three traps, all measured:

- `gh issue view` does not print the type, and `--json type` is not a field. It is `--json issueType`.
- The minimized state is absent from the single comment endpoint and present on the list, so the check reads the list.
- `gh issue view --comments` omits a minimized comment with no flag to include it, so a reader who needs the dead text reads the API.

The false negative floor is stated rather than estimated.
Five issues carry a body edit with no rename and no mark, and a body edit without a rename is indistinguishable from an ordinary addition.

One property is never decidable: whether a span is still true is a judgement.

**This is an agreement rather than a gate, carried where the actor already reads.**
An agreement holds when it reaches whoever is about to act: the same instruction block in the prototype ran at 83.9% on one carrier and at nothing on another.

| carrier | reaches | state |
|---|---|---|
| `AGENTS.md` | the agent, before it acts | owed by [the ticket that writes it](https://github.com/kalonji-tools/purba/issues/46), which is blocked on this record |
| the review thread `.github/workflows/sign.yml` posts | the reviewer while they decide, blocking the merge until resolved | live |
| an issue or pull request template | the web interface and nothing else | 0 of 1,317 issues and 0 of 899 pull requests in the prototype |

A template is rendered by the client that opens it and every issue here arrived through the API, the reason [a register belongs to one location](a-register-belongs-to-one-location.md) already gives.
`CONTRIBUTING.md` fails from the other side, addressing outside contributors this project has never had.

The reviewer judges what no check can, which is the reader [architectural significance is declared, not detected](significance-is-declared-not-detected.md) already relies on.
