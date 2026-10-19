# Command lends one parse to selection and assertion

## Context and Problem Statement

Selection and assertion both need the AST of a selected module, and no other part needs the tree.
[purba parses Python with ruff](purba-parses-python-with-ruff.md#decision-outcome) makes one parser, and says nothing about how often it parses.

| fact | source |
|---|---|
| ruff parse of loguru's 50 test modules | 3.48 ms, [the measurement](https://github.com/kalonji-tools/purba/issues/580#issuecomment-6179050920) |
| CPython `compile()` of the same modules | 44.9 ms, [the measurement](https://github.com/kalonji-tools/purba/issues/580#issuecomment-6179050920) |
| assertion's rewrite of the same modules | about 6% of a ruff parse timed beside it, [the rewrite measurement](https://github.com/kalonji-tools/purba/issues/580#issuecomment-6222966987) |
| live bytes of a kept tree and its source | 10.0 times the bytes of the source, [the measurement](https://github.com/kalonji-tools/purba/issues/580#issuecomment-6179050920) |
| the parser leaves every `NodeIndex` empty, and no AST node is serializable | [How do tools that parse Python once let every later stage use the same AST?](https://github.com/kalonji-tools/purba/issues/578#issuecomment-6176414882) |
| only selection and assertion use the tree. Execution, fixtures and reporting want a fact from it in depth, and command and dispatch want nothing | [the findings](https://github.com/kalonji-tools/purba/issues/580#issuecomment-6185440346) |
| a crate that names its consumers and is named by them makes a cycle, which Cargo refuses | [the findings](https://github.com/kalonji-tools/purba/issues/580#issuecomment-6185440346) |
| execution can pass a database on while it names only assertion's interface | [the findings](https://github.com/kalonji-tools/purba/issues/580#issuecomment-6185440346) |

## Considered Options

- **Each part asks parsing for the module, and parsing reads on each request.** Rejected. Parsing parses each module twice, from a file that can change between the two reads. Each worker needs a parser.
- **The tree travels to each part that wants it.** Rejected. No ruff node crosses a process. Execution, fixtures and reporting would depend on ruff's types for one fact each.
- **Parsing keeps each tree for the run.** Rejected. A kept tree holds far more than its source, to save a parse that costs a fraction of `compile()`.
- **Parsing keeps the text, so each file is read once.** Rejected. One parse before the run already gives each part one text.
- **Selection reads the file and hands the text to parsing.** Rejected. A second part then opens test files, and parsing parses a text it did not read.
- **A database with one interface for each part.** Rejected. `&dyn Db` passes through every call, and a run whose order is fixed repays no database.
- **One walk calls each consumer registered with parsing.** Rejected. Assertion sees each module before selection decides, which breaks lazy work.
- **Selection calls assertion's rewrite.** Rejected. Selection then holds two jobs.
- **A node travels as a `NodeIndex`, or as a kind and a range.** Rejected. The parser leaves every index empty, and a later part wants a fact, not a node.
- **Assertion changes a copy of the tree.** Rejected. A tree goes back to source only through one more ruff crate, or a Python `ast` that assertion builds across the seam.
- **Command lends one parse to selection and then to assertion's rewrite, and only facts cross a part.** Chosen.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**Command lends one parse of each module to selection, and then to assertion's rewrite.**

| question | answer |
|---|---|
| who reads a test file | parsing, when command asks for the module by its path |
| who holds the tree, and how long | command, for one module, while selection and then assertion's rewrite read it |
| what crosses from one part to another | facts only: each selected test, the rewritten text of its module, and the text of each assert, as plain data |
| when assertion rewrites | before the run, and only a module where selection selected a test |
| how assertion rewrites | it splices the source text at node ranges, and never changes the tree |
| a worker in another process | it gets the rewritten text, so it needs no parser |

[A worker loads each module once, and command hands each module once](a-worker-loads-each-module-once.md#decision-outcome) holds how often a module crosses and loads.

**Downside:**

- **Assertion rewrites every selected module before the first test runs.** The rewrite costs a small share of the parse. The run holds each rewritten text, about the size of its source, until that module loads.
- **Command uses parsing and assertion.** A change to either interface edits command.
- **Assertion works at two times.** Its rewrite runs before the run, and its load runs when execution imports the module.

## Confirmation

No command checks it, because purba holds no product code.
[Add purba-parsing, which parses a module with ruff](https://github.com/kalonji-tools/purba/issues/567), [Add purba-selection, which names the tests of a module by their identity](https://github.com/kalonji-tools/purba/issues/568), [Add assertion's rewrite, which marks each assert so that it records that it ran](https://github.com/kalonji-tools/purba/issues/569) and [Add purba-command, which turns the arguments into an exit code](https://github.com/kalonji-tools/purba/issues/574) build it.
