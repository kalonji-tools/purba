# purba records delegation and does not prevent it

## Context and Problem Statement

An agent here runs on the machine that holds a person's credentials.
[Liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md) removed the approval on a deployment environment.
That approval was the one act here an agent could not perform.
An agent that holds a person's credentials can now approve as that person.

So the question is whether any act by a person can be made impossible for their agent to forge.

## Considered Options

- **A signature.** Rejected. A key kept where the agent runs is a key the agent uses, so a signature proves custody rather than personhood.
- **A credential out of the agent's reach.** Rejected. The agent runs on the machine that holds the credentials.
- **The block GitHub documents on an approval of one's own pull request.** Rejected. It keys on the author rather than on the approver.
- **A hardware key that demands a touch for each signature.** Rejected. It is genuinely out of reach, and git cannot ask for one. The allowed-signers format `ssh-keygen(1)` documents admits `cert-authority`, `namespaces`, `valid-after` and `valid-before`. None of them concerns the presence of a person.
- **Record the delegation, and imply no separation.** Chosen. Every route above is closed. A record that claimed one would claim a check nobody makes.

## Decision Outcome

**purba records delegation and does not prevent it.**
An agent acts for a person here, and that person remains answerable for what it does.
This record says so, and implies no separation that no check makes.

**So the question is not whether a trailer can be forged, but whether the procedure preserves the intent it records.**
git's own answers to frequently asked questions refuse a `commit.signoff` setting for that reason.
They hold that an automated sign-off would let someone argue later that the trailer was added out of habit rather than to certify anything.
`scripts/sign-branch.sh` is the answer this project already holds.
It refuses a caller with no terminal, prints every commit and who wrote it, and waits to be answered.
It cannot tell who typed the answer, and it claims no more than that.

**Downside:**

- **No check separates a person from their agent.** The agent runs where the credentials live, so no mechanism reachable here can make one.
- **A trailer records an act and never who performed it.** An agent can run `git commit -s` as easily as a person can.
- **`git commit -s` run in the clone where the agent works writes the agent's sign-off.** That clone's git config names the agent as the committer. `CONTRIBUTING.md` addresses a contributor whose committer identity is already their own. That is how this repository is worked rather than a defect in the document, and nothing here repairs it.

## Confirmation

No command decides this, and none can.

[Liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md) grades `none` on the rows that bear on it: who wrote the origin trailer, and whether a person is distinguishable from their agent.

Each route above was measured rather than reasoned.
[An act by a person cannot be made unforgeable here, because the agent holds the human's credentials](https://github.com/kalonji-tools/purba/issues/204) records every measurement beside the source it rests on.
