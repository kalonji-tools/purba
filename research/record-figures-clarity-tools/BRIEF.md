# Sweep brief: which records would a figure make clearer?

You are one of six agents sweeping the decision records of purba (a Rust + Python test runner).
Your batch is a list of record filenames. Read EVERY record in it, whole.

## Paths

- S = <scratchpad>
- Records (snapshot of `origin/main` at `78232b7`, read-only): `$S/sweep/before/<file>.md`
- The whole source tree at `78232b7`, read-only: `$S/tree/` (Rust crates under `$S/tree/crates/` or similar; `ls` it)
- Your batch list: `$S/sweep/batch-<N>.txt`
- Do NOT touch any git checkout, and do not commit or push anything.
- You may not be able to write files. Do not rely on it. Your FINAL MESSAGE is the deliverable.

## The question

For each record: **would a figure make some part of it clearer to a person reading it?**

The word count decides NOTHING. Do not count words, and do not argue from length. A figure that adds words but reads clearer is a candidate. A figure that saves words but reads no clearer is not.

The person who reads these records is a visual reader. A figure earns its place where the content's SHAPE is a relation that the prose spells out in a line, and the reader must rebuild that shape in their head:

- a sequence of messages between parts over time
- a flow with branches, or a state machine
- a tree, a containment, or a dependency graph
- a timeline
- an equation or an inequality, written as a sentence
- a curve, where its shape decided the question

Argument is not a picture. A list of reasons, a rejected option, a rationale, a Downside: these are rarely clearer as a figure. A table that is already a clear matrix is rarely clearer as a diagram. Do not force a candidate. A record with none is the normal outcome.

## The rules that bind (read these records whole first)

- `$S/sweep/before/a-figure-in-a-record-is-a-fence-github-renders.md` decides the FORM:
  diagram = `mermaid` fence; formula = `math` fence; plot = `mermaid` `xychart`, only where its shape decided the question, and in place of the table so each number appears once.
  No icons (except `cloud database disk internet server` in an `architecture` diagram). No `$` math outside a `math` fence.
- `$S/sweep/before/an-artifact-holds-the-minimum-that-conveys-its-point.md` lines 71-73: "A diagram enters where it reads clearer than the words it replaces. A reviewer decides whether a diagram reads clearer."

## The bar: five figures already accepted

These five records got a figure on `main` in the last pull request. Read at least two before you start, to see the bar:
`a-worker-loads-each-module-once.md`, `reporting-writes-each-line-as-dispatch-hands-it-the-test.md`, `a-line-names-its-kind-in-the-glossarys-words.md`, `command-lends-one-parse-to-selection-and-assertion.md`, `a-design-in-depth-waits-for-what-it-assumes.md` (all under `$S/sweep/before/`).

An earlier sweep judged records by word count. Its drafts are in `$S/sweep/prior-787.md`. Four of its drafts were not picked, and some sit in your batch: `an-outside-contribution-is-applied-not-merged.md`, `an-unpublished-comment-carries-what-no-other-location-carries.md`, `purba-parses-python-with-ruff.md`, `purba-runs-inside-the-testers-interpreter.md`. Judge them fresh, by clarity. You may reuse or improve their draft.

## What a candidate must hold

1. **The figure says what the replaced span says. No claim added, no claim dropped.** If the prose names no order, the figure names none.
2. **An arrow's ORDER and a loop's nesting are claims.** Where a figure draws a sequence of what purba does, check each arrow against the CODE in `$S/tree/`, not only against the records. Cite the file and line that sets the order. In the last pull request, two figures drew an order the code does not make. If the code does not exist yet (a planned part), say so, and draw only the order the records state.
3. **Name what the figure loses.** A Mermaid label holds no Markdown link, so a link to the glossary (`CONTEXT.md#…`) or to another record is lost unless a sentence outside the fence keeps it. A code span, bold, or an exact term may be lost too. List each loss.
4. **The fence parses** under the Mermaid GitHub runs today (12.2.0). Run the parser; never judge by eye. A `math` fence must render under MathJax.
5. A sentence outside the span may change only to point at the figure, or to keep a link the figure loses. Quote each changed sentence.

Mermaid traps: never use a node id that is a Mermaid keyword (`call`, `click`, `end`, `style`, `class`, `href`, `callback`, `default`, `graph`, `subgraph`, `classDef`, `linkStyle`); quote a label holding `()[]{}|:;#"`. In a `sequenceDiagram`, a participant named `end` or `loop` breaks it; use an alias.

## Tools (run them)

Pipe a Markdown snippet holding the fence to the checkers on stdin (`-`), so you need not write a file:

```
printf '%s\n' '```mermaid' 'flowchart LR' '  a --> b' '```' | nix-shell -p nodejs --run "node $S/tools/parse-fences.mjs $S/mmparse-deps -" 2>&1 | grep -v ExperimentalWarning
printf '%s\n' '```math' 'a \le b' '```' | nix-shell -p nodejs --run "node $S/tools/render-math.mjs $S/mathjax-deps -" 2>&1 | grep -v ExperimentalWarning
```

Each prints `ok`/`FAIL` per fence. Controls: a node named `call` fails with `CALLBACKNAME`; an unclosed `aligned` fails with `Missing \end{aligned}`. If a heredoc or `printf` is refused, say so in your final message; the coordinator parses every fence again anyway.

## Your final message: exactly this shape

First a TSV block, one line per record in your batch (every record, so the sweep shows it read all of them):

````
```tsv
<file.md>	none
<file.md>	candidate	<k>	<L>-<M>	diagram|formula|plot	strong|borderline
```
````

`<L>-<M>` are the 1-indexed, inclusive line numbers of the replaced span in `$S/sweep/before/<file.md>`. `k` = 1, 2, … if one record has several independent candidates. `strong` = you think a person would read it clearer; `borderline` = it may, the person should judge.

Then one section per candidate, exactly:

````
### <file.md> --<k>

**Replaces:** lines <L>-<M>: <ten words or fewer naming what those lines are>
**Kind:** diagram | formula | plot
**Parsed:** ok (Mermaid 12.2.0) | ok (MathJax) | not run: <why>

The replaced span, verbatim:

> <each replaced line, quoted>

The figure:

```mermaid
<the figure>
```

**Shows that the words do not:** <one or two sentences>
**Loses:** <each link, code span or term the figure drops, or "nothing">
**Order checked against:** <file:line for each arrow order, or "no order drawn", or "planned part, records only: <record>">
**Prose outside the fence that changes:** <each new sentence quoted, or "none">
````

Then one line naming any record you were unsure about, and why.
