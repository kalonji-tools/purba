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

Two Mermaid features fail on GitHub with no error.
`layout: elk` falls back to the default layout.
An unknown icon pack draws a near-empty image.

## Considered Options

- **A committed image, drawn with a tool GitHub does not render, such as mingrammer's `diagrams` or draw.io.** Rejected. Nobody can review its diff. The image goes stale when nobody renders it again.
- **A fence GitHub renders from its source.** Chosen. Each figure is text in the diff. GitHub draws it for every reader.

## Decision Outcome

**Reach:** `docs/decisions/*.md`

**A figure in a record is a fence GitHub renders.**

| kind | fence | rule |
|---|---|---|
| diagram | `mermaid` | never `layout: elk`, and never an icon pack |
| formula | `math` | never `$` math outside the fence |
| plot | `mermaid`, as an `xychart` | only where its shape decided the question, and in place of the table, so each number appears once |

A graph is a diagram.
A fence is not [prose](../../CONTEXT.md#prose), so the prose rules do not read the text inside it.

**Downside:**

- **Nothing refuses a fence before the merge.** A fence that does not parse draws an error box for every reader after it lands.
- **GitHub picks the Mermaid version, and announces no change to it.** A diagram that renders now can break with no change to the record.
- **A figure that neither fence can draw has no form here.** It waits for a new record.

## Confirmation

| property | check |
|---|---|
| each figure takes the fence its kind names | a reviewer |
| a `mermaid` fence parses | GitHub draws an error box in place of the diagram |

[Which of the figure rules does lint:records refuse, and does a gate parse each Mermaid fence?](https://github.com/kalonji-tools/purba/issues/458) decides what a gate takes.
