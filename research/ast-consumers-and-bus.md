# Which parts would use a parsed AST, and how many stages can share one parse

Read on 2026-10-15. Each tool is pinned in [Sources](#6-sources).
The draft under review is `docs/decisions/a-part-asks-parsing-for-its-ast.md` at [`88f5116`][draft].
**READ** marks a claim checked in the source. **INFERENCE** marks my reasoning. **PROBE** marks a throwaway Cargo workspace I compiled in the scratchpad.

## 1. What was found

Only selection and assertion need the tree itself at the walking skeleton.
Three more parts would want a static fact in depth: reporting, execution and fixtures.
Each of those three wants a small fact, not a tree.
Reporting wants a statement range. Execution and fixtures want parameter names and types.
A fact like that is plain data, so it can travel inside a selected test or an outcome.
Command and dispatch have no use for an AST. Real dispatchers schedule by the test's identity string.
Every "register with the parser" pattern found in real tools keeps one owner of the walk.
Ruff, pylint, libcst, flake8 and Sphinx all do this.
That owner either names each consumer, or a wiring root hands the consumers to it.
In Rust the first form makes a dependency cycle, and Cargo refuses it (PROBE).
The second form keeps the existing arrows to parsing, and adds arrows from one wiring root to each subscriber.
A registry filled at link time (`inventory`) adds no arrow, so the diagram check cannot see the wiring.
All bus patterns fire when the module is parsed.
For assertion that is the wrong time, because assertion works when execution imports the module, possibly in another process.
A ruff tree cannot reach that process.
So a bus saves the second parse only in a single-process run, and only if assertion's work moves to selection time.
The draft's database rejection ("dispatch and execution then use a parsing type") is false as worded.
ty layers one `Db` trait per crate, and a probe compiled execution passing the store while naming only assertion's trait.
The real costs of a database lie elsewhere, and section 4 lists them.

## 2. Which part would use an AST

Skeleton: `purba test_a.py`, three tests (`assert 1 == 1`, `assert 1 == 2`, `pass`), and a second module that is never imported.

| part | uses the AST at the skeleton | would use it in depth | could get the fact otherwise, and the cost | evidence |
|---|---|---|---|---|
| command | **No.** It reads argv and sets the exit code | **No** (INFERENCE). Configuration is TOML, and the exit code comes from outcomes | n/a | landscape [L55-L65][landscape] |
| selection | **Yes.** It finds each module-level `def test_*` and gives it an identity | **Yes.** It needs classes, decorators and parametrize for the query, which is the same visitor grown larger | Import the module and read `__dict__`. **Cost:** an import, which breaks "imports none of them" and lazy work. The module's code runs | rtest collects with `ruff_python_parser::parse_module` and a `visit_stmt` over `FunctionDef` and `ClassDef` [rt-disc] [rt-vis]. pytest imports and reads `__dict__` [pt-py] (READ) |
| parsing | owns the parse | owns the parse | n/a | [ruff decision][ruffrec] |
| dispatch | **No** | **No** (INFERENCE). A grouping such as "by module" or "by class" can be read from the identity, or carried as a fact in the selected test | n/a. A tree cannot cross to a worker process anyway: no ruff AST node is serializable [prior research] | pytest-xdist groups by splitting the node id string [xd-scope]. Its workers send node id lists back, and the controller compares those lists [xd-remote] [xd-load]. rtest's scheduler takes `Vec<String>` node ids [rt-sched] [rt-run] (READ) |
| execution | **No.** It gets the index of the last assert from the call, and the assert's text from assertion | **A fact, not the tree.** It needs the names and types of a test's parameters to ask fixtures for them | `inspect.signature` on the live function. **Cost:** execution already imports the module, so no extra import. But the live function sits on the Python side of the seam, and Rust must cross the seam to read the parameters | pytest reads parameter names at run time with `signature(function)` [pt-compat]. pytest's own TODO says fixtures "should ask the node for the fixture names, and not try to obtain directly from the function object well after collection" [pt-compat] (READ) |
| fixtures | **No.** It builds nothing at the skeleton | **Maybe** (INFERENCE). To keep lazy work, it could find which module defines the fixture for a type without importing every candidate module | Import every module that holds fixtures and scan its attributes. **Cost:** imports outside the selection | pytest scans `dir(holderobj)` of imported modules [pt-fix] (READ). ruff finds a fixture statically from its decorator, `pytest.fixture`, on every `FunctionDef` [rf-fixture] [rf-stmt-fix] (READ) |
| assertion | **Yes.** It finds each `assert`, its range and its text, and splices `_purba_ran(i, …)` in | **Yes.** It also needs sub-expression ranges, to show values | None. The rewrite must exist before the import, so a live module is too late | pytest parses at import and rewrites only selected or matching files [pt-should] [prior research PT4, PT6] (READ) |
| reporting | **A fact, as data.** The failed line quotes the assert's text, which it receives through the outcome | **Yes** (INFERENCE). For an error raised in a helper module, it shows the whole failing statement, and that module was never selected or parsed | A traceback gives one line number. **Cost:** a multi-line statement is cut short. The traceback is a live object, which the seam record refuses to execution and which "an outcome is data" keeps from reporting | pytest calls `ast.parse` on the frame's whole file at report time to find the statement range [pt-src]. It caches that tree per report [pt-code] (READ) |

INFERENCE, from the table: a bus that runs at selection time does not serve reporting's need in depth.
The helper module is a file that selection never read.
Only a request by path, as the draft chose, or a database, serves an arbitrary file at an arbitrary time.

## 3. The patterns, named for what they do

The legend for the arrows:

```
existing (landscape):   selection -> parsing      assertion -> parsing
                        command -> selection | dispatch | reporting
                        dispatch -> selection | execution
                        execution -> fixtures | assertion      reporting -> execution
```

### 3.1 One walk calls every consumer that registered for a node kind

The walker visits each node once.
For each node kind it calls the handlers registered for that kind.

- **pylint (READ):** `ASTWalker.add_checker` reads each checker's `visit_<kind>` methods into `visit_events[kind]` [pl-walk]. `walk` calls them per node [pl-walk2]. `PyLinter` is the wiring root: it calls `add_checker` for every checker [pl-root].
- **libcst (READ):** `visit_batched` merges the `visit_<Type>` methods of many visitors into one `_BatchedCSTVisitor` [lc-batch]. `on_visit` calls each one by the node's type name [lc-onvisit].
- **CPython (READ):** `NodeVisitor.visit` dispatches on `'visit_' + node.__class__.__name__` [cp-ast]. This is the one-visitor base that pylint and libcst batch.
- **ruff (READ):** `Checker::visit_stmt` calls `analyze::statement` [rf-visit]. That function is a hand-written `match` on the statement kind [rf-stmt]. Each arm calls each rule behind `checker.is_rule_enabled(Rule::…)`. For example, the `Stmt::Assert` arm calls the bandit, pyflakes, bugbear and pytest-style rules [rf-assert]. The rules are modules of the same crate (`use crate::rules::{…}`) [rf-stmt]. So ruff's walker names every consumer, and no crate boundary separates them.

Two Rust forms for purba:

```rust
// FORM A — parsing names each consumer (ruff's shape, across crates)
// purba-parsing
pub fn walk(m: &ModModule) {
    for stmt in &m.body {
        match stmt {
            Stmt::FunctionDef(f) => purba_selection::on_function(f), // parsing -> selection
            Stmt::Assert(a)      => purba_assertion::on_assert(a),   // parsing -> assertion
            _ => {}
        }
    }
}
```

**Arrows:** this INVERTS the arrows. It adds parsing → selection and parsing → assertion beside the existing selection → parsing.
**PROBE:** Cargo refuses it with `error: cyclic package dependency: package purba-parsing … depends on itself`.
Form A works only if the walker and the rules share one crate, as in ruff.

```rust
// FORM B — parsing owns the trait; a root registers the subscribers
// purba-parsing
pub trait OnModuleParsed { fn on_module(&mut self, path: &str, m: &Module); }
pub fn parse_and_publish(path: &str, src: String, subs: &mut [&mut dyn OnModuleParsed]) {
    let m = parse(path, src);
    for s in subs.iter_mut() { s.on_module(path, &m); }
}   // the tree drops here
// purba-selection:  impl purba_parsing::OnModuleParsed for Selection { … }
// purba-assertion:  impl purba_parsing::OnModuleParsed for Rewrites  { … }
// the root:
purba_parsing::parse_and_publish(path, src, &mut [&mut selection, &mut rewrites]);
```

**Arrows:** the existing selection → parsing and assertion → parsing stay as they are.
Form B ADDS root → parsing, root → selection and root → assertion.
**PROBE:** this compiled, and its test passed: one parse, two subscribers.
`cargo tree` showed command → {assertion, parsing, selection}.
If command is the root, command → parsing and command → assertion are new arrows.
Today assertion is reached only through execution.

**Costs:**
- Only subscribers carry a parsing type. Dispatch and execution carry none.
- The tree lives one call, so memory is the same as one parse.
- **Worker in another process (INFERENCE):** the publish happens where selection runs. Assertion's subscriber would get the tree in the main process. Its product must then travel to the worker as data, for example `Vec<(TextRange, String)>` edits, inside a selected test. Otherwise the worker parses again, and the bus saved nothing.
- **Lazy work (INFERENCE):** the landscape says assertion rewrites "as it is imported". A parse-time subscriber computes a rewrite for each module selection reads. In depth, a query that reads a directory also reads modules that it then selects nothing from. Computing a rewrite for those modules is arguably a rewrite outside the selection.
- **One identity, one parser:** both hold. Selection still mints identities, and one parser runs.
- **The diagram is the only wiring:** it holds only if the diagram draws the root's new arrows.

### 3.2 Each consumer declares what it needs, and the owner computes each product once, only if asked

This is a refinement of 3.1.
A consumer declares a product, such as the tree, the tokens or a derived table.
The owner computes only the products that some enabled consumer declared, and computes each one once.

- **libcst (READ):** a visitor declares `METADATA_DEPENDENCIES` [lc-dep]. `MetadataWrapper.resolve_many` gathers providers transitively. It computes each provider once, batching those that can share a walk, and keeps the result in `wrapper._metadata` [lc-wrap] [lc-resolve].
- **ruff (READ):** each `Rule` returns a `LintSource`: `Ast`, `Tokens`, `PhysicalLines`, `Imports` and others [rf-src]. `check_path` runs the token, AST, import and line checkers only when an enabled rule needs that source [rf-path].
- **flake8 (READ):** a plugin declares its input by naming a parameter, such as `tree`, `logical_line` or `physical_line`. flake8 reads the parameter names with `inspect.signature` [f8-params] and sorts the plugins by them [f8-classify]. `run_ast_checks` builds the AST once and calls every `tree` plugin with it [f8-ast].

```rust
// purba-parsing
pub enum Product { Tree, AssertRanges, TestDefs }
pub trait Consumer { const NEEDS: &'static [Product]; fn take(&mut self, p: &Products); }
pub struct Products { /* filled only for what some consumer NEEDS */ }
```

**Arrows:** the same as 3.1 Form B. Consumers → parsing exist, and the root → each consumer is new.
**What it adds:** the parse produces no product that no consumer asked for.
It fits purba's "lazy work" in spirit, but lazy work is about imports, and this is about computation.
**Worker in another process (INFERENCE):** the products are plain data, unlike the tree. So `AssertRanges` and `TestDefs` can cross a process. This is the one bus variant whose output survives the worker boundary.

### 3.3 Any part asks a database, which parses on first request and memoizes

- **ty and ruff_db (READ, partly from prior research):** the query `parsed_module(db, file)` memoizes the parse, with an LRU [prior research D1, D2].
- **ty layers the database trait by crate (READ):**
  - `ruff_db::Db: salsa::Database` [ty-rdb]
  - `ty_python_core::Db: ModuleResolverDb` [ty-core]
  - `ty_python_semantic::Db: PythonCoreDb` [ty-sem]
  - The one concrete `ProjectDatabase` in `ty_project` implements each trait, one `impl` per crate [ty-proj] [ty-proj-impl].
- **rust-analyzer:** the same shape [prior research RA4, RA5].

```rust
// purba-parsing
pub trait Db { fn parsed(&self, path: &Path) -> Arc<Module>; }
// purba-assertion  (assertion -> parsing, exists)
pub trait Db: purba_parsing::Db {}
pub fn rewrite(db: &dyn Db, path: &Path) -> String { /* db.parsed(path) */ }
// purba-execution  (execution -> assertion, exists; never names purba-parsing)
pub trait Db: purba_assertion::Db + purba_fixtures::Db {}
pub fn run(db: &dyn Db, test: &Selected) -> Outcome { purba_assertion::rewrite(db, …) }
// purba-dispatch: pub trait Db: purba_execution::Db + purba_selection::Db {}
// the root:  struct PurbaDb { … }  impl purba_parsing::Db for PurbaDb { … }  impl purba_assertion::Db for PurbaDb {}  …
```

**Arrows:** every part's trait extends only the traits of parts it already uses, so no part gains an arrow.
The root that implements every trait gains one arrow to each part.
Today the `purba` crate "uses purba-command alone" [crate-rec].
**PROBE:** this compiled, and its test passed. The test made one parse for two asks, one from a selection stand-in and one from execution through assertion.
`purba-execution`'s manifest and source never name `purba-parsing`. `cargo tree -p purba-execution` showed execution → assertion → parsing.
The upcast from `&dyn execution::Db` to `&dyn assertion::Db` compiled. That upcast is stable Rust.

**Costs:**
- Every call down the stack carries `&dyn Db`, and every part defines a `Db` trait.
- A salsa database adds the salsa dependency, and `ruff_db`'s pin moves with ruff [prior research D1].
- **Worker in another process:** the worker builds its own database and parses again. ty runs in one process with threads [prior research T5].
- **Lazy work:** this holds, because nothing is parsed until a part asks.
- **One parser:** this holds.
- **The diagram is the only wiring:** it holds if the diagram draws the root's arrows.
- INFERENCE: the store could be plain memoization behind parsing's own API. Parts would then pass nothing at all, at the price of global state.

### 3.4 Parsing publishes "module parsed", and subscribers registered on a bus receive it

- **Sphinx (READ):** extensions call `EventManager.connect(name, callback, priority)` [sx-connect]. `emit` calls the listeners in priority order [sx-emit]. The transform `DoctreeReadEvent` emits `doctree-read` with the parsed document [sx-read]. The event list describes it as "the doctree before being pickled" [sx-list].
- **Sphinx in parallel (READ):** each reader process parses its own documents and emits the events there. It then pickles `self.env` back to the main process, which merges it [sx-par]. A Sphinx doctree can be pickled. A ruff AST cannot.
- **pytest (READ):** pluggy hooks such as `pytest_collect_file(file_path, parent)` publish a path, not a tree [pt-hook]. Each plugin opens and parses the file itself.
- **Rust registration at link time (READ):** `inventory::collect!(T)` in the crate that owns `T`, and `inventory::submit!` in any downstream crate. All submissions "take effect simultaneously" before `main`, or at `dlopen` [inv].

```rust
// purba-parsing
pub struct Subscriber { pub on_module: fn(&Path, &Module) }
inventory::collect!(Subscriber);
pub fn parse(path: &Path) -> Module {
    let m = /* ruff */;
    for s in inventory::iter::<Subscriber> { (s.on_module)(path, &m) }
    m
}
// purba-selection:  inventory::submit! { purba_parsing::Subscriber { on_module: selection::on_module } }
// purba-assertion:  inventory::submit! { purba_parsing::Subscriber { on_module: assertion::on_module } }
```

**Arrows:** only the existing subscriber → parsing arrows. No root is needed.
**Costs (INFERENCE):**
- The wiring exists, but no manifest shows it. The check in [each part is a crate][crate-rec] reads `cargo metadata`, so it cannot see who receives the tree. That is the hidden-wiring breach this invariant exists to stop.
- A subscriber is a `&'static` value with a `fn` pointer. Its results must go into global state, which is an owner that no diagram shows.
- The order of calls is unspecified [inv].
- It shares 3.1's timing and worker problems: the event fires at selection time, in selection's process.
- Not READ: whether `inventory` registers inside a PyO3 `cdylib` loaded by `import`. The README says that elements in a dynamically loaded library register "at the time that dlopen occurs" [inv]. No probe was built.

### 3.5 Each consumer parses again, and only its product travels (the draft's choice)

- **flake8 (READ):** its process pool gets only filenames, `pool.imap_unordered(_mp_run, self.filenames)`. Each worker parses its own file [f8-pool].
- **pytest-xdist (READ):** each worker collects, which means import and parse, then sends `ids=[item.nodeid …]` [xd-remote]. The controller refuses differing collections [xd-load].
- **rtest (READ):** its Rust static collector makes node-id strings for the scheduler [rt-run] [rt-sched]. Its native worker re-imports each file and re-discovers the tests with `inspect.getmembers` [rt-worker]. INFERENCE: that is two collections that can disagree, which is purba's risk "text selection disagrees with import".
- **pytest, libcst, Pyright:** see the prior research [PT3, LC4, PY5].

**Arrows:** the existing ones, unchanged.
**Worker:** this works in any process.
**Cost:** one parse per consumer.

### 3.6 Sequential passes over one tree that one owner keeps

- **Pyright (READ):** `SourceFile.bind` constructs a `Binder`, which is a `ParseTreeWalker`, and calls `bindModule(parseTree)`. Later, `check` constructs a `Checker`, also a `ParseTreeWalker`, and calls `check()` [py-bind] [py-check] [py-binder] [py-checker]. The `SourceFile` keeps the tree between the passes.

This is the prior research's "the caller that parses lends the tree".
It needs one owner that calls every pass, and purba's diagram has none.
INFERENCE: it also needs every pass to run in the owner's process.

### Which patterns add arrows, and which invert them

| pattern | new arrows | inverted arrows | a parsing type outside the subscribers | survives a worker process |
|---|---|---|---|---|
| 3.1 A, the walker names each consumer | — | parsing → selection, parsing → assertion. **Cargo refuses the cycle** (PROBE) | no | no |
| 3.1 B, a root registers subscribers | root → parsing, root → assertion (if the root is command) | none | no | only if products travel as data |
| 3.2, declared products | as 3.1 B | none | no | yes for products, no for the tree |
| 3.3, a database with layered traits | root → every part | none | no; each part names only its own `Db` (PROBE) | no; the worker builds its own database |
| 3.4, a link-time bus (`inventory`) | none visible | none | no | no |
| 3.5, parse again | none | none | no | yes |

## 4. What this changes in the draft's rejections (all INFERENCE)

| draft option | its stated reason | verdict | the reason that holds |
|---|---|---|---|
| the caller that parses lends the tree | no part calls both selection and assertion | **stands, with a different reason** | Selection runs at selection time. Assertion runs at import time, and a worker may import in another process. No caller spans both, and a ruff tree cannot cross the process. Drawing a new arrow would not fix either |
| a store passed down, or ruff_db's database | dispatch and execution then use a parsing type | **falls as worded** | ty layers one `Db` trait per crate, and the PROBE compiled execution naming only assertion's trait. The real costs: the root gains an arrow to every part; `&dyn Db` runs through every call; salsa, if used; and no saving across a process |
| parsing keeps each tree for the run | memory is 10 times the source | stands | measured, and it does not rest on the diagram |
| parsing keeps the text | purba writes no test file | stands | it does not rest on the diagram |
| selection reads the file and hands the text to parsing | assertion cannot reach that text | **weak** | Text is a `String` and can cross a process inside the selected test. Its value is one read, which the row above already judged low. Reject it on that ground instead |
| a node travels as a `NodeIndex`, or as a kind and a range | the parser leaves every index empty, and no skeleton part needs a node | **half stands** | The `NodeIndex` half stands. The kind-and-range half needs no index (`covering_node`, prior research A7). The real reason is the second clause. Section 2 shows that the later consumers want facts, not node handles |
| assertion changes a copy of the tree | one more crate, or a Python `ast` | stands | it does not rest on the diagram |
| **missing:** one parse publishes to registered subscribers (3.1 B, 3.2, 3.4) | not considered | **needs a row** | It saves the parse only in one process, and only if assertion's work moves to selection time. It needs a root with new arrows (3.1 B, 3.2), or wiring that no manifest shows (3.4) |

The chosen option, a request by path, is the only one that also serves reporting's need in depth.
That need is a statement range in a module nobody selected.
It also serves a worker without any data travelling with the test.
The "Downside: parsing parses the file again for each part that asks" is the measured cost, at 3.48 ms for 50 modules.

## 5. What could not be verified

- Whether `inventory` or `linkme` registration works inside a maturin-built `cdylib` that Python loads. Not probed.
- Whether a salsa database can be built per worker cheaply. ty's single-process model was read, and no multi-process salsa user was read.
- mypy's passes (`build.py`) were not read.
- Pyright's callers of `getParseResults()` were not read (as in the prior research).
- The probes used stand-in types, not ruff's `Parsed<ModModule>`. The prior research's probe showed `Parsed<ModModule>: Send + Sync + 'static`, so an `Arc` of it fits the database sketch. That claim was not re-run here.
- The claim that fixtures in depth would use static facts rests on lazy work and ruff's static detection. No test runner was found that resolves fixtures statically.
- Ruff's rules do sit in the same crate as the walker. I did not check whether ruff has ever split them, or why it has not.

## 6. Sources

Pinned commits:

| source | ref | commit |
|---|---|---|
| ruff and ty | as in the prior research | `5b27c1ccfaa230fb195855f5cf3404a21e3379cd` |
| pytest | 9.1.1 | `cf470ec0bf7eb89cd97dd56df4859eae5db46447` |
| libcst | v1.9.0 | `c029c17bf45a3737fc8d1347001ab2422f42ae58` |
| pylint | v4.1.3 | `e75de10c69103381c875601f638d4b977ff0fcaf` |
| flake8 | 7.4.1 | `7eccf20b526fe4f2230bf9afeeef2fc9f97faf8d` |
| Sphinx | v9.1.0 | `cc7c6f435ad37bb12264f8118c8461b230e6830c` |
| pytest-xdist | v3.8.0 | `1e3e4dc16523c8a8f6c67d95a950166420718c99` |
| rtest | v0.0.47 | `b9eddee626457fef3dbfd4ccbe425867cec4d20c` |
| CPython | v3.14.0 | `ebf955df7a89ed0c7968f79faec1de49f61ed7cb` |
| inventory | 0.3.25 | `ffc228473d7aa20ca0790ddccc894a20682489ae` |
| Pyright | 1.1.414 | `a28cce4077d87b80bead12de809f79845e39c8ba` |
| purba `main` | | `97bac7694e760de82e10b0845059dbf1a7394ff9` |
| the draft record | | `88f5116a09c0cf28ea0b6cda183f7b4b9bdb4b26` |
| the prior research | `research/ast-sharing` | `72de47bd684dfad19af0bd979c206805a1ccf624` |

The probes are in this scratchpad, under `probe/ws` (3.1 B), `probe/ws-cycle` (3.1 A) and `probe/db` (3.3). They ran with cargo 1.101.0-nightly, 2026-10-07, through `mise exec`.

[draft]: https://github.com/kalonji-tools/purba/blob/88f5116a09c0cf28ea0b6cda183f7b4b9bdb4b26/docs/decisions/a-part-asks-parsing-for-its-ast.md
[landscape]: https://github.com/kalonji-tools/purba/blob/97bac7694e760de82e10b0845059dbf1a7394ff9/LANDSCAPE.md#L55-L95
[ruffrec]: https://github.com/kalonji-tools/purba/blob/97bac7694e760de82e10b0845059dbf1a7394ff9/docs/decisions/purba-parses-python-with-ruff.md#decision-outcome
[crate-rec]: https://github.com/kalonji-tools/purba/blob/97bac7694e760de82e10b0845059dbf1a7394ff9/docs/decisions/each-part-is-a-crate.md#decision-outcome
[prior research]: https://github.com/kalonji-tools/purba/blob/72de47bd684dfad19af0bd979c206805a1ccf624/research/ast-sharing.md

- [rf-visit] `visit_stmt` calls `analyze::statement`: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/checkers/ast/mod.rs#L1048 and #L1767
- [rf-stmt] `use crate::rules::{…}` and `match stmt`: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/checkers/ast/analyze/statement.rs#L1-L20
- [rf-assert] the `Stmt::Assert` arm: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/checkers/ast/analyze/statement.rs#L1150-L1180
- [rf-stmt-fix] fixture rules on `FunctionDef`: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/checkers/ast/analyze/statement.rs#L269-L287
- [rf-fixture] `is_pytest_fixture`: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/rules/flake8_pytest_style/helpers.rs#L32-L42
- [rf-src] `LintSource` and `Rule::lint_source`: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/registry.rs#L233-L252
- [rf-path] `check_path` runs only the checkers some rule needs: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/linter.rs#L153-L258
- [ty-rdb] https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_db/src/lib.rs#L74-L80
- [ty-core] https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_python_core/src/db.rs#L7-L12
- [ty-sem] https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_python_semantic/src/db.rs#L8-L21
- [ty-proj] `ProjectDatabase`: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_project/src/db.rs#L90-L92
- [ty-proj-impl] one `impl` per layer: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_project/src/db.rs#L596-L600 and #L655-L656
- [pt-src] `getstatementrange_ast` calls `ast.parse`: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/_code/source.py#L181-L193
- [pt-code] `TracebackEntry.getsource` with `astcache`: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/_code/code.py#L285-L310
- [pt-compat] `getfuncargnames`, its TODO, and `signature(function)`: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/compat.py#L108-L137
- [pt-fix] `parsefactories` scans `dir(holderobj)`: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/fixtures.py#L2104-L2110
- [pt-py] collection reads `__dict__`: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/python.py#L400-L411
- [pt-should] `_should_rewrite`: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L229-L248
- [pt-hook] `pytest_collect_file`: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/hookspec.py#L365
- [lc-batch] `BatchableCSTVisitor`, `visit_batched`, `_get_visitor_methods`: https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/_batched_visitor.py#L29-L101
- [lc-onvisit] `_BatchedCSTVisitor.on_visit`: https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/_batched_visitor.py#L125-L135
- [lc-dep] `METADATA_DEPENDENCIES` and `resolve`: https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/_metadata_dependent.py#L72-L110
- [lc-wrap] `_gather_providers`, `_resolve_impl`: https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/metadata/wrapper.py#L56-L95
- [lc-resolve] `resolve`, `resolve_many`, `visit_batched`: https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/metadata/wrapper.py#L170-L215
- [pl-walk] `ASTWalker.add_checker`: https://github.com/pylint-dev/pylint/blob/e75de10c69103381c875601f638d4b977ff0fcaf/pylint/utils/ast_walker.py#L28-L70
- [pl-walk2] `ASTWalker.walk`: https://github.com/pylint-dev/pylint/blob/e75de10c69103381c875601f638d4b977ff0fcaf/pylint/utils/ast_walker.py#L72-L92
- [pl-root] `PyLinter` registers every checker: https://github.com/pylint-dev/pylint/blob/e75de10c69103381c875601f638d4b977ff0fcaf/pylint/lint/pylinter.py#L1093-L1103 and the walk at #L1224
- [f8-params] `_parameters_for`: https://github.com/PyCQA/flake8/blob/7eccf20b526fe4f2230bf9afeeef2fc9f97faf8d/src/flake8/plugins/finder.py#L262-L286
- [f8-classify] `_classify_plugins`: https://github.com/PyCQA/flake8/blob/7eccf20b526fe4f2230bf9afeeef2fc9f97faf8d/src/flake8/plugins/finder.py#L310-L335
- [f8-ast] `run_ast_checks`: https://github.com/PyCQA/flake8/blob/7eccf20b526fe4f2230bf9afeeef2fc9f97faf8d/src/flake8/checker.py#L377-L396
- [f8-pool] the pool gets filenames: https://github.com/PyCQA/flake8/blob/7eccf20b526fe4f2230bf9afeeef2fc9f97faf8d/src/flake8/checker.py#L203
- [cp-ast] `NodeVisitor.visit`: https://github.com/python/cpython/blob/ebf955df7a89ed0c7968f79faec1de49f61ed7cb/Lib/ast.py#L482-L506
- [sx-list] https://github.com/sphinx-doc/sphinx/blob/cc7c6f435ad37bb12264f8118c8461b230e6830c/sphinx/events.py#L59
- [sx-connect] https://github.com/sphinx-doc/sphinx/blob/cc7c6f435ad37bb12264f8118c8461b230e6830c/sphinx/events.py#L363-L375
- [sx-emit] https://github.com/sphinx-doc/sphinx/blob/cc7c6f435ad37bb12264f8118c8461b230e6830c/sphinx/events.py#L405-L445
- [sx-read] `DoctreeReadEvent`: https://github.com/sphinx-doc/sphinx/blob/cc7c6f435ad37bb12264f8118c8461b230e6830c/sphinx/transforms/__init__.py#L418-L424
- [sx-par] `_read_parallel` pickles the env back: https://github.com/sphinx-doc/sphinx/blob/cc7c6f435ad37bb12264f8118c8461b230e6830c/sphinx/builders/__init__.py#L592-L626
- [xd-remote] workers send node ids: https://github.com/pytest-dev/pytest-xdist/blob/1e3e4dc16523c8a8f6c67d95a950166420718c99/src/xdist/remote.py#L256-L261
- [xd-load] `_send_tests`, `_check_nodes_have_same_collection`: https://github.com/pytest-dev/pytest-xdist/blob/1e3e4dc16523c8a8f6c67d95a950166420718c99/src/xdist/scheduler/load.py#L302-L325
- [xd-scope] `_split_scope`: https://github.com/pytest-dev/pytest-xdist/blob/1e3e4dc16523c8a8f6c67d95a950166420718c99/src/xdist/scheduler/loadscope.py#L284-L306
- [rt-disc] `discover_tests` calls `parse_module`: https://github.com/hughhan1/rtest/blob/b9eddee626457fef3dbfd4ccbe425867cec4d20c/src/python_discovery/discovery.rs#L119-L130 (the ruff crates: https://github.com/hughhan1/rtest/blob/b9eddee626457fef3dbfd4ccbe425867cec4d20c/Cargo.toml#L21-L24)
- [rt-vis] `TestDiscoveryVisitor::visit_stmt`: https://github.com/hughhan1/rtest/blob/b9eddee626457fef3dbfd4ccbe425867cec4d20c/src/python_discovery/visitor.rs#L69-L84
- [rt-sched] `distribute_tests`, `extract_scope`: https://github.com/hughhan1/rtest/blob/b9eddee626457fef3dbfd4ccbe425867cec4d20c/src/scheduler.rs#L41-L60 and #L95-L103
- [rt-run] `execute_tests_parallel(…, test_nodes: Vec<String>)`: https://github.com/hughhan1/rtest/blob/b9eddee626457fef3dbfd4ccbe425867cec4d20c/src/runner.rs#L52-L66
- [rt-worker] the native worker re-discovers by import: https://github.com/hughhan1/rtest/blob/b9eddee626457fef3dbfd4ccbe425867cec4d20c/python/rtest/worker/runner.py#L162-L190 and #L313-L320
- [inv] `collect!`, `submit!`, life-before-main, `dlopen`, order: https://github.com/dtolnay/inventory/blob/ffc228473d7aa20ca0790ddccc894a20682489ae/README.md?plain=1#L30-L108
- [py-bind] https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/analyzer/sourceFile.ts#L905-L935
- [py-check] https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/analyzer/sourceFile.ts#L990-L1018
- [py-binder] https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/analyzer/binder.ts#L254
- [py-checker] https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/analyzer/checker.ts#L218
