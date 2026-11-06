# A figure in a record is a fence GitHub renders

## Context and Problem Statement

A reader opens a record on GitHub, and no build step runs before that.
[An artifact holds the minimum that conveys its point](an-artifact-holds-the-minimum-that-conveys-its-point.md) admits a diagram where it carries the point, and says nothing of its form.
The design of purba needs three kinds of figure: a diagram, a formula and a plot.

GitHub renders some syntaxes from their source in Markdown.
It strips inline SVG.

| syntax | GitHub renders it from source |
|---|---|
| Mermaid | yes |
| math, written in LaTeX | yes |
| D2, Graphviz, PlantUML, draw.io | no |

GitHub registers no Mermaid icon pack, and its stylesheet holds no FontAwesome.
So an icon draws nothing and raises no error, unless it is a built-in icon of an `architecture` diagram.

## Considered Options

- **A committed image, drawn with a tool GitHub does not render, such as mingrammer's `diagrams` or draw.io.** Rejected. Nobody can review its diff. The image goes stale when nobody renders it again.
- **A fence GitHub renders from its source.** Chosen. Each figure is text in the diff. GitHub draws it for every reader.

## Decision Outcome

**Reach:** `docs/decisions/*.md` `scripts/check-records.sh` `scripts/check-prose/**` `scripts/test/check-records.bats`

**A figure in a record is a fence GitHub renders.**

| kind | fence | rule |
|---|---|---|
| diagram | `mermaid` | never an icon, except `cloud`, `database`, `disk`, `internet` and `server` in an `architecture` diagram |
| formula | `math` | never `$` math outside the fence |
| plot | `mermaid`, as an `xychart` | only where its shape decided the question, and in place of the table, so each number appears once |

A graph is a diagram.
A fence is not [prose](../../CONTEXT.md#prose), so the prose rules do not read the text inside it.

**Downside:**

- **No gate parses a fence.** A gate would pin a Mermaid version that GitHub does not run, and so decide a different question. A reviewer sees a fence that does not parse only on the branch, as an error box.
- **GitHub picks the Mermaid version, and announces no change to it.** Its Mermaid moved from 11.17.2 to 12.1.0 in one month. Version 12.0 made ELK the default layout. A merged diagram took a new layout and a new look with no edit to the record.
- **A figure that neither fence can draw has no form here.** It waits for a new record.

## Confirmation

| property | check |
|---|---|
| each figure takes the fence its kind names | a reviewer |
| a `mermaid` fence parses | the reviewer opens the record on the branch, where GitHub draws it |

`lint:records` refuses the math rule and the icon rule.
`scripts/test/check-records.bats` and the tests inside `scripts/check-prose/check-prose.rs` run each refusal green and red.

| refusal | green | red |
|---|---|---|
| `$` math | `$var` in a code span, a `math` fence and a price written with two dollar signs pass | inline math and a `$$` block are refused at their line, in a heading and a link title too |
| an icon | the five built-in icons of an `architecture` diagram pass, and so does an icon outside a `mermaid` fence | an icon from a pack in an `architecture` diagram, an `icon:` shape, a FontAwesome label and a mindmap `::icon(` are each refused at their line |
