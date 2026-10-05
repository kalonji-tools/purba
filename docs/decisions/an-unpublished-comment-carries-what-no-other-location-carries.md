# An unpublished comment carries what no other location carries

## Context and Problem Statement

Five locations record why a change is the way it is, and they repeat each other.

| location | what states its register |
|---|---|
| commit message | [a commit message outlives its review](a-commit-outlives-its-review.md) |
| decision record | [a decision record is rewritten, not amended](a-record-is-rewritten-not-amended.md) |
| pull request | `.github/PULL_REQUEST_TEMPLATE.md` |
| published comment | the language's convention for voice, and [a register belongs to one location](a-register-belongs-to-one-location.md) for which artifact earns one |
| unpublished comment | nothing, and this record ends that |

The fifth is the only one with nothing saying what belongs there, so a writer with something to say puts it in all five.

Volume is not the defect.

| corpus | non-blank | unpublished comment | share |
|---|---:|---:|---:|
| prototype Rust and Python production | 71,382 | 3,959 | 5.5% |
| this repository | 343 | 176 | 51.3% |
| this repository, `src/lib.rs` | 17 | 0 | 0% |

Every unpublished comment in this repository sits in configuration.

Duplication is the defect.
`Cargo.toml` holds nine comment blocks over twenty-seven lines.

| block | lines | restates |
|---|---:|---|
| `crate-type` | 6 | [the crate carries an rlib](the-crate-carries-an-rlib.md) |
| the `ruff` pins | 4 | [purba parses Python with ruff's parser](purba-parses-python-with-ruff.md) |
| `extension-module` | 3 | [the crate carries an rlib](the-crate-carries-an-rlib.md) |

Each of the three cites the closed issue that produced the record, so the pointer reaches what happened rather than what is true.

## Considered Options

- **Locality: a comment explains the line it sits on.** Rejected. All three duplicated blocks pass it, because each is local to the line it sits on.
- **A comment-to-code ratio.** Rejected. Removing the duplicated lines improves `Cargo.toml` and raises every ratio computable over it.
- **A separate register for configuration.** Rejected. The split between configuration and code counts violations rather than a difference in kind, and the admission test below names configuration.
- **Trap-only: a comment exists to stop a wrong edit.** Rejected as the whole rule. Whether an edit is plausible is a judgement. It survives inside the admission test.
- **Subtraction: a comment carries what the other locations do not.** Chosen. It removes the duplicated lines by construction.

## Decision Outcome

An unpublished comment is written only when the information belongs in no other location.

**The order in which a location takes information.**
The pull request, then the commit and the issue its subject names, then the decision record, then the published comment.
What remains may be an unpublished comment.
Each of those four states in its own register what it takes.
This record restates none of them.

**Information on the issue is already in the commit, so it is not written again.**
A decision record subtracts more strongly, because it sits in the same tree and states what is true.

**Admission.**
A comment is written when complex configuration, logic or an algorithm needs elaboration that no other location holds.

**A link replaces the elaboration wherever a primary source exists.**
The research paper, the published algorithm, the upstream reference, the framework and the design pattern each stay correct without maintenance.

**A comment cites that source and never an issue number.**
The issue number belongs to the commit subject.

**Fewer unpublished comments is a goal this project works towards.**
It is not a threshold, and no check counts them.

**This record is the exception to [a register belongs to one location](a-register-belongs-to-one-location.md).**
That record calls this register convention, and names what this section gates.
Link liveness and the paths a comment cites are decidable, so those two are gated and the rest is convention.

**Downside:**

- **The rule that decides is unenforceable.** Whether information belongs in another location is a judgement, so nothing gates the sentence this record exists to state.
- **The recommended citation is the form most likely to fail the gate.** A digital object identifier resolves to a publisher, and a publisher blocks the checker. The gate refuses such a link until the change that adds it also adds an exception and its reason.
- **A checked link is not a read link.** The gate proves a page answers, never that it still says what the comment claims.
- **The online leg depends on a network.** It refuses a change for the state of somebody else's server, so it reads only the files a change touches.
- **A dead link in a file that no change touches stays dead.** The next change to that file meets it. Its author then repairs a link they did not write.

## Confirmation

**`Quality` runs `mise run lint:links` on every pull request.**

| leg | reads | refuses |
|---|---|---|
| offline | every tracked file that `.gitattributes` does not mark generated | a link to a file or a heading in the tree that does not exist |
| online | the files a branch changes against its base, generated files left out | an address on the network that does not answer |
| readme | the file `pyproject.toml` names as `readme`, which PyPI shows as purba's page | a link that names neither a full address nor a heading of that file |
| cited paths | each comment line outside Markdown and generated files | a path under `docs/`, `scripts/`, `src/`, `.github/` or `.config/` that git does not track |

`lychee.toml` holds the settings the checker reads in each leg that runs it, with the reason for each beside it.
The script passes the flags that differ per leg.
`scripts/test/check-links.bats` builds each shape the gate refuses offline.
A fragment that quotes text, an `http` link with an `https` form and a redirect act on the network alone, and no test reaches them.

The checker reads a file it is given by name, whatever its extension.
Outside Markdown and HTML it finds a full address only, so a cited path needs a leg of its own.
It extracts a bare address, a titled link and an autolink alike.
It resolves a relative link between records to a path it then checks.

**Blind spots.**

| blind spot | effect |
|---|---|
| reserved example domains are excluded by default | a placeholder address is not a checked address |
| a root file cited with no directory, such as `mise.toml` | a rename of that file leaves the citation standing |
| a path in a Markdown code span | a rename leaves the path standing |
| a link to a heading of the README that names the README, such as `README.md#building` | PyPI resolves it to nothing, and the readme leg passes it, because the checker reads it as a heading of the same file |

Nothing decides whether a comment that survives the subtraction was worth writing.
That fails as friction.
