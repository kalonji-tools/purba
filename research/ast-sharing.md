# How tools that parse Python once let every later stage use the same AST

Research for [#578](https://github.com/kalonji-tools/purba/issues/578).
Read on 2026-10-14 against the pinned commits and versions in [Sources](#sources).
This note surveys patterns and does not choose one: a grilling issue chooses.

## What was found

None of the tools read hands one AST through callers that have no use for it.
Each tool does one of two things.
Some keep one owner that every stage asks, either a stack frame or a query database.
Others parse again where the tree is needed and cache the product, not the tree.
The query tools (ty, rust-analyzer) treat the tree itself as disposable.
What travels between their stages is a small, plain handle: a node index, or a node kind plus a byte range.
A stage turns that handle back into a node against whatever tree the owner holds now.
Stages that rewrite code mostly splice source text at node ranges, and leave the tree unchanged.
pytest is the exception: it mutates Python's own `ast` in place and compiles the tree.
For purba this matters in three ways.
First, `ruff_python_ast` 0.0.17 nodes own their data and carry a `NodeIndex` slot.
The parser leaves that slot empty, and only `ruff_db` fills it.
Second, no AST node in that crate is serializable, so a tree cannot cross a process boundary.
A worker process gets an AST only by parsing again.
Third, selection and assertion both already point at parsing.
A store that parsing owns, or a plain handle, reaches both without a new arrow.

## The five questions, tool by tool

| tool | 1. who owns the AST, and how long it lives | 2. how a later stage refers to a node | 3. how a rewriting stage changes code | 4. how the AST reaches a stage the parser does not call | 5. across processes or workers |
|---|---|---|---|---|---|
| **ruff's linter** | A local `Parsed<ModModule>` in `lint_only` owns it, for one file's check [L2] | A borrow `&'a Stmt`, held by `Checker` [L4]. `SemanticModel` keeps an arena of borrows indexed by `NodeId` [L6]. Deferred work stores `&'a Expr` plus a snapshot [L7] | Splices text. A fix is an `Edit` of a range and new text [L9]. `apply_fixes` copies the text between edits [L8]. Then the loop parses again, at most 100 times [L3] | It does not. `check_path` takes `&Parsed` and calls every checker itself [L1]. One traversal visits the tree [L5] | `par_iter` parses each file on its own thread [L10]. The disk cache stores only whether a file linted clean, never a tree [L11] |
| **ty** | The salsa query `parsed_module` owns a `ParsedModule`, with an LRU of 200 [D1]. `clear()` drops the tree, and `load()` parses again on demand [D2]. A checked file that is not open is cleared [T5] | `AstNodeRef<T>`: a `NodeIndex` and a type marker [T1]. `node(&ParsedModuleRef)` resolves it [T2] | ty's checker does not rewrite code. ty's editor actions were not read | Every query calls `parsed_module(db, file).load(db)` itself [T4]. The database is the channel | One process. rayon threads share the database [T5]. A type holding `AstNodeRef` must not cross modules: deserializing it "requires reparsing the entire module" [T3] |
| **rust-analyzer** | The salsa query `parse` owns a green tree, with an LRU of 128 [RA4] [RA3]. "Syntax trees are transient objects" [RA1] | `SyntaxNodePtr`: a node kind and a text range [RW1]. `AstPtr<N>` adds the node type [RA2]. Items get a `FileAstId` from kind, name and parent, so it survives edits elsewhere [RA6] [RA7] | Builds a new tree. `SyntaxEditor` clones the subtree and records changes [RA8]. The old and new trees are diffed into range edits [RA9] [RA10] | Every query asks the database for `parse` or `ast_id_map` [RA5], then resolves its pointer against that root [RA2] | One process with threads. A marker inside `ErasedFileAstId` "is used by the proc-macro server" [RA11]. What else crosses that process boundary was not read |
| **pytest** | A local in `_rewrite_test`: `ast.parse`, rewrite, `compile`, then the tree is garbage [PT4] | Nothing refers back. A failure report parses the source again [PT10], with a per-report `astcache` dict [PT11]. Assert text is found again by tokenizing, keyed by line [PT7] | Mutates in place. `setattr(node, name, new)` replaces statement lists [PT6]. `compile()` accepts the AST object [CP1] | It does not. Collection imports the module [PT8] and reads its `__dict__` [PT9]. The import runs the meta path hook [PT1], which reads and parses the file itself [PT3] | Every process rewrites at its own import. The rewritten code is cached as a `.pyc` tagged with the pytest version [PT2]. It is checked by mtime and size [PT5], and written atomically for "concurrent pytest processes" [PT3] |
| **Pyright** | Each `SourceFile` owns its parser output [PY4]. When memory is low, `dropParseAndBindInfo` drops it, and it is parsed again when needed [PY3] | The node object itself, which carries `id` and `parent` [PY1]. Binder facts live in a `WeakMap<ParseNode, AnalyzerNodeInfo>` beside the tree [PY2] | Splices text. An edit is a range and replacement text [PY6] | Through `SourceFile.getParseResults()` [PY4]. The callers were not read | A background thread receives the file's text as a message, not a tree [PY5] |
| **libcst** | The caller owns an immutable `Module`. `MetadataWrapper` deep-copies it by default [LC2] | Node identity. Metadata maps a node to a value, so the copy ensures no node appears twice [LC2] | Copies. Nodes are frozen dataclasses [LC1]. `with_changes` builds a new node [LC1] | The wrapper visits the tree for each visitor. A provider's metadata is computed on first request, then kept [LC2] | Codemod workers are separate processes. Each one reads and parses its own file [LC4] |

## What `ruff_python_ast` 0.0.17 gives purba

purba's `Cargo.toml` on `main` names no ruff crate yet.
The decision record still states the pin `=0.0.12`.
This note reads `=0.0.17`, as [#578](https://github.com/kalonji-tools/purba/issues/578)'s brief named.
The 0.0.17 tarballs of `ruff_python_ast`, `ruff_python_parser` and `ruff_db` record commit `5b27c1cc` in `.cargo_vcs_info.json`.
Their files match that commit byte for byte, for every file cited below.

| question | answer | where |
|---|---|---|
| Does a node carry an index? | Yes. Every node struct has `node_index: AtomicNodeIndex` beside `range: TextRange` | [A3] [A4] [A5] |
| What is the index? | `NodeIndex(NonZeroU32)`, "a unique index for a node within an AST". The top two bits mark a string-annotation sub-AST | [A1] |
| Does the parser fill it? | No. The parser writes `AtomicNodeIndex::NONE` into every node | [P3] |
| Who fills it? | `ruff_db`'s private indexing visitor numbers nodes in source order | [D5] |
| Can purba fill it? | `AtomicNodeIndex::set` is public and takes `&self`. It is "interiorly mutable to allow assigning node indices on-demand after parsing" | [A2] |
| Can purba look a node up by index? | Not in `ruff_python_ast`. `get_by_index` lives in `ruff_db`, behind a private module and a private constructor | [D3] [D4] [D6] |
| Can purba look a node up by range? | Yes. `find_node::covering_node(root, range)` returns the smallest node that contains a range | [A7] |
| Do nodes own their data? | Yes. Children sit in `Box`, `ThinVec` and owned names. Only view types such as `AnyNodeRef<'a>` borrow | [A4] [A6] |
| Does `Parsed` own its tree? | Yes. `Parsed<T>` holds the tree, the tokens and the errors, and derives `Clone` | [P1] |
| Is a parsed module shareable across threads? | Yes. The probe below compiled `Parsed<ModModule>: Send + Sync + 'static` | probe |
| Can a tree cross a process? | No. The `serde` feature derives only on `PySourceType`, `Name` and `PythonVersion`, never on a node | [A8] |
| Is `ruff_db` on crates.io? | Yes, `0.0.17`, from `astral-sh/ruff`. Its `parsed_module` needs a salsa database | [D1] |

A stable reference therefore exists, with a condition.
A `NodeIndex` names one node within one parse, once something has numbered that parse.
Numbering is a source-order walk, so the same text gives the same numbers.
That last point is an **inference** from the visitor's order [D5]; no test of it was read.
An edit anywhere earlier in the file shifts every later index.

The probe was a throwaway binary against the three crates at `=0.0.17`.
It parsed a three-statement test module and counted the statements whose `node_index` was `NONE`.
It printed `statements=3 with NodeIndex::NONE=3`.
It then moved an `Arc<Parsed<ModModule>>` into another thread and read the tree there.

## The patterns, named for what they do

Each sketch uses purba's parts.
`purba-parsing` returns an AST, and `purba-selection` and `purba-assertion` both use it.
The sketches show shapes, not designs, and do not choose.

### The caller that parses lends the tree to every stage it calls

ruff's linter parses once and passes `&Parsed` down one call tree [L1] [L5].
Every stage borrows; nothing outlives the caller's frame.

```rust
// one caller holds both stages
let parsed: Parsed<ModModule> = purba_parsing::parse(path, &source)?;
let tests: Vec<Selected> = purba_selection::select(&parsed, &query);
let code: String = purba_assertion::rewrite(&parsed, &source);
```

- **What it needs:** one caller that calls both stages.
- **What it costs purba:** the diagram has no such caller. Command calls selection; execution calls assertion.
- **What it gives:** no copy, no cache, and no lifetime beyond one frame.

### Parsing keeps each module and hands every caller the same shared tree

ty and rust-analyzer memoize the parse in a database query [D1] [RA4].
Any stage asks for it by file, and gets a cheap, shared handle [D2] [RA3].
The store may drop a tree and parse it again on the next request [D2] [PY3].

```rust
// purba-parsing
pub struct Module { source: Box<str>, parsed: Parsed<ModModule> }
pub struct Modules { /* path and content hash -> Arc<Module> */ }

impl Modules {
    /// Parses on the first request, then returns the same tree.
    pub fn get(&self, path: &Path) -> Result<Arc<Module>, Diagnostic>;
}

// purba-selection and purba-assertion each call `modules.get(path)`
```

- **What it needs:** both stages reach the same `Modules` value.
- **What it costs purba:** that value is a parsing type. A value passed down the call tree crosses command, dispatch and execution. A global avoids the crossing, but hides an owner. ty passes its database to every query as `&dyn Db` [D1] [T4].
- **What it gives:** no new arrow, because both stages already point at parsing.

### A node travels as a number, and a stage resolves it against the tree

ty stores `AstNodeRef<T>`, a `NodeIndex` and a type marker [T1].
A stage resolves it with the tree that the owner holds now [T2].
Pyright and libcst hang later facts in a side table keyed by the node [PY2] [LC2].

```rust
// plain data: no parsing type crosses a part
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId { pub file: FileId, pub index: u32 }

// purba-parsing numbers nodes once, after parsing
impl Module {
    pub fn node(&self, index: u32) -> AnyNodeRef<'_>;
}

// a later stage keeps facts beside the tree
let failed: HashMap<NodeId, AssertFact>;
```

- **What it needs:** something numbers the nodes, because the parser leaves `NONE` [P3]. The lookup table is purba's to build [D6].
- **What it costs purba:** the number means nothing without the same text. A changed file shifts it.
- **What it gives:** a `Copy` value that passes through dispatch and execution, and through a process boundary.

### A node travels as a kind and a byte range, and a stage finds it again

rust-analyzer's `SyntaxNodePtr` is a kind and a range [RW1].
It finds the node by descending from the root, in about logarithmic time [RW1].
`ruff_python_ast` 0.0.17 already ships the descent as `covering_node` [A7].

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeAt { pub file: FileId, pub kind: NodeKind, pub range: TextRange }

// any stage that holds the tree
let node = ruff_python_ast::find_node::covering_node(module.parsed.syntax().into(), at.range);
```

- **What it needs:** nothing beyond the published crate. No numbering pass.
- **What it costs purba:** a lookup is a walk, not an index. A changed file breaks the range, so a content hash must travel with it.
- **What it gives:** the same handle survives a second parse of the same text, in any process.

### Each process parses again, and caches what it made, not the tree

pytest parses at import, in whatever process imports [PT3].
It caches the rewritten code as a `.pyc`, checked by mtime and size [PT5].
ruff caches a verdict [L11]. libcst and Pyright workers receive a path or text, never a tree [LC4] [PY5].

```rust
// purba-assertion, in the process that imports the module
pub fn rewrite(path: &Path) -> Result<Rewritten, Diagnostic> {
    let module = purba_parsing::parse_file(path)?; // the second parse, same parser
    // ...
}

// the cache key names the source, not the tree
struct CacheKey { path: PathBuf, mtime: SystemTime, size: u64, purba: &'static str }
```

- **What it needs:** nothing shared between stages.
- **What it costs purba:** one more parse for each selected module. It is the shape the prototype of [#564](https://github.com/kalonji-tools/purba/issues/564) already has.
- **What it gives:** it keeps "one parser, not one parse", and it survives any worker model.

### A rewrite splices source text at node ranges, and never touches the tree

ruff, rust-analyzer and Pyright all end a rewrite as range edits on text [L9] [RA10] [PY6].
ruff applies them and parses the result again [L3] [L8].
pytest instead mutates the tree in place [PT6], and libcst builds a new tree [LC1].

```rust
pub struct Edit { pub range: TextRange, pub replacement: Box<str> }

// purba-assertion reads the shared tree and writes new text
pub fn rewrite(module: &Module, asserts: &[NodeAt]) -> String; // edits sorted by range
```

- **What it needs:** the source text beside the tree, since nodes hold offsets, not text.
- **What it costs purba:** a new node cannot be read back from the tree without a parse.
- **What it gives:** the tree stays immutable, so an `Arc` can share it. purba's seam record already compiles rewritten source, not a tree [S2].

## Trade-offs to carry into the grilling

- **The arrows decide which handle can travel.** Dispatch hands execution "one test", and execution gets "a rewritten module" from assertion [S1]. A value on that path that names a parsing type needs an arrow the diagram does not draw [S3]. A `u32` index or a `TextRange` does not.
- **The worker model decides whether a tree can travel at all.** Dispatch hands a test to "whatever the worker is" [S1]. A tree cannot leave its process, because no node is serializable [A8].
- **A handle is only as stable as the text under it.** An index or a range from selection is valid in assertion only if the file has not changed. rust-analyzer's `FileAstId` survives edits by naming the item, not its position [RA6].
- **Indices cost a numbering pass, and a lookup table.** `ruff_db` builds both, privately, with `unsafe` address arithmetic [D3] [D4]. A borrowing table would avoid `unsafe` but carries a lifetime. That alternative is an **inference**, not read anywhere.

## What this note could not verify

- What rust-analyzer sends to its proc-macro server process, beyond the marker comment [RA11].
- Which Pyright stages call `getParseResults()`; only the accessor was read [PY4].
- Whether ty's editor features rewrite code; only the checker was read.
- pytest-xdist was not read. The claim about many processes rests on pytest's own comment [PT3].
- That the same text always gets the same `NodeIndex` numbers. It follows from a source-order walk, but no test of it was read.

## Sources

Pinned versions and commits:

| source | version | commit |
|---|---|---|
| `ruff_python_ast`, `ruff_python_parser`, `ruff_db` | 0.0.17 on crates.io | `5b27c1ccfaa230fb195855f5cf3404a21e3379cd` |
| ruff and ty | the same ruff commit | `5b27c1ccfaa230fb195855f5cf3404a21e3379cd` |
| rust-analyzer | tag `2026-10-12` | `2ad81e72b7e978cb800dffe1bcce5c9f98032acf` |
| rowan | 0.17.0 on crates.io, the version rust-analyzer locks | `c781c336f44a28210fefaa4d86b8c9902b781c8e` |
| pytest | tag `9.1.1` | `cf470ec0bf7eb89cd97dd56df4859eae5db46447` |
| Pyright | tag `1.1.414` | `a28cce4077d87b80bead12de809f79845e39c8ba` |
| libcst | tag `v1.9.0` | `c029c17bf45a3737fc8d1347001ab2422f42ae58` |
| Python docs | 3.14 | |
| purba | `main` | `499a8c8d777921b4e035ee35e4ff208dbc7e1bf7` |

`ruff_python_ast` 0.0.17:

- [A1] `NodeIndex`: [node_index.rs#19-61](https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/node_index.rs.html#19-61)
- [A2] `AtomicNodeIndex`, interior mutability and `set`: [node_index.rs#156-180](https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/node_index.rs.html#156-180)
- [A3] `ModModule`: [generated.rs#9277-9284](https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/generated.rs.html#9277-9284)
- [A4] `StmtFunctionDef`: [generated.rs#9299-9311](https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/generated.rs.html#9299-9311)
- [A5] `StmtAssert`: [generated.rs#9476-9483](https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/generated.rs.html#9476-9483)
- [A6] `AnyNodeRef<'a>`, a borrowing view: [generated.rs#5831](https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/generated.rs.html#5831)
- [A7] `covering_node`: [find_node.rs#7-14](https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/find_node.rs.html#7-14)
- [A8] the only `serde` derives: [lib.rs#98-100](https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/lib.rs.html#98-100), [name.rs#36-38](https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/name.rs.html#36-38), [python_version.rs#180-195](https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/python_version.rs.html#180-195)

`ruff_python_parser` 0.0.17:

- [P1] `Parsed<T>`: [lib.rs#385-392](https://docs.rs/ruff_python_parser/0.0.17/src/ruff_python_parser/lib.rs.html#385-392)
- [P3] the parser writes `AtomicNodeIndex::NONE`: [parser/mod.rs#225-229](https://docs.rs/ruff_python_parser/0.0.17/src/ruff_python_parser/parser/mod.rs.html#225-229)

`ruff_db` 0.0.17:

- [D1] the `parsed_module` query: [parsed.rs#19-43](https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#19-43)
- [D2] `ParsedModule`, `load` and `clear`: [parsed.rs#107-162](https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#107-162)
- [D3] `ParsedModuleRef::get_by_index`: [parsed.rs#189-206](https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#189-206)
- [D4] `IndexedModule::new` and its lookup: [parsed.rs#572-622](https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#572-622)
- [D5] the indexing visitor: [parsed.rs#624-650](https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#624-650)
- [D6] the private `mod indexed` and private `new`: [parsed.rs#120](https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#120), [parsed.rs#228](https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#228)

ty:

- [T1] `AstNodeRef`: [ast_node_ref.rs#L13-L56](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_python_core/src/ast_node_ref.rs#L13-L56)
- [T2] `AstNodeRef::new` and `node`: [ast_node_ref.rs#L74-L114](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_python_core/src/ast_node_ref.rs#L74-L114)
- [T3] "module-local type": [expression.rs#L111-L139](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_python_core/src/expression.rs#L111-L139)
- [T4] `semantic_index` loads the parse itself: [lib.rs#L74-L83](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_python_core/src/lib.rs#L74-L83)
- [T5] parallel check, then `clear`: [ty_project/src/lib.rs#L545-L576](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_project/src/lib.rs#L545-L576)

ruff's linter:

- [L1] `check_path` takes `&Parsed`: [linter.rs#L121-L135](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/linter.rs#L121-L135)
- [L2] `lint_only` parses: [linter.rs#L446-L457](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/linter.rs#L446-L457)
- [L3] the fix loop, at most `MAX_ITERATIONS`: [linter.rs#L374](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/linter.rs#L374), [linter.rs#L560-L645](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/linter.rs#L560-L645)
- [L4] `Checker` borrows `parsed`: [checkers/ast/mod.rs#L196-L234](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/checkers/ast/mod.rs#L196-L234)
- [L5] `check_ast`, one traversal: [checkers/ast/mod.rs#L3552-L3631](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/checkers/ast/mod.rs#L3552-L3631)
- [L6] `Nodes<'a>` and `NodeId`: [ruff_python_semantic/src/nodes.rs#L16-L87](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_semantic/src/nodes.rs#L16-L87)
- [L7] deferred visits: [checkers/ast/deferred.rs#L8-L16](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/checkers/ast/deferred.rs#L8-L16)
- [L8] `apply_fixes`: [fix/mod.rs#L54-L121](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/fix/mod.rs#L54-L121)
- [L9] `Edit`: [ruff_diagnostics/src/edit.rs#L12-L17](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_diagnostics/src/edit.rs#L12-L17)
- [L10] `par_iter` over files: [commands/check.rs#L83](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff/src/commands/check.rs#L83)
- [L11] the cache holds a verdict: [cache.rs#L335-L361](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff/src/cache.rs#L335-L361)

rust-analyzer and rowan:

- [RA1] "syntax trees are transient": [ptr.rs#L1-L10](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/syntax/src/ptr.rs#L1-L10)
- [RA2] `AstPtr`: [ptr.rs#L21-L65](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/syntax/src/ptr.rs#L21-L65)
- [RA3] `Parse<T>` holds a green tree: [lib.rs#L69-L98](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/syntax/src/lib.rs#L69-L98)
- [RA4] the `parse` query, `lru = 128`: [editioned_file_id.rs#L20-L28](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/base-db/src/editioned_file_id.rs#L20-L28)
- [RA5] the `ast_id_map` query: [hir-expand/src/lib.rs#L1752-L1757](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-expand/src/lib.rs#L1752-L1757)
- [RA6] why `FileAstId` stays stable: [ast_id.rs#L1-L21](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/span/src/ast_id.rs#L1-L21)
- [RA7] `FileAstId`, `AstIdMap`, `get`: [ast_id.rs#L286-L291](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/span/src/ast_id.rs#L286-L291), [ast_id.rs#L600-L609](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/span/src/ast_id.rs#L600-L609), [ast_id.rs#L842-L846](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/span/src/ast_id.rs#L842-L846)
- [RA8] `SyntaxEditor` clones the subtree: [syntax_editor.rs#L30-L49](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/syntax/src/syntax_editor.rs#L30-L49)
- [RA9] old and new trees diffed into a text edit: [source_change.rs#L290-L292](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/ide-db/src/source_change.rs#L290-L292)
- [RA10] `Indel` and `TextEdit`: [text_edit.rs#L18-L29](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/ide-db/src/text_edit.rs#L18-L29)
- [RA11] the proc-macro server marker: [ast_id.rs#L44-L54](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/span/src/ast_id.rs#L44-L54)
- [RW1] rowan's `SyntaxNodePtr`, kind and range: [ast.rs#50-87](https://docs.rs/rowan/0.17.0/src/rowan/ast.rs.html#50-87)

pytest:

- [PT1] the hook joins `sys.meta_path`: [assertion/\_\_init\_\_.py#L129-L133](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/__init__.py#L129-L133)
- [PT2] `PYTEST_TAG`: [rewrite.py#L68-L70](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L68-L70)
- [PT3] `exec_module`, the cache and concurrent processes: [rewrite.py#L148-L188](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L148-L188)
- [PT4] `_rewrite_test`: [rewrite.py#L343-L351](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L343-L351)
- [PT5] `_read_pyc` checks mtime and size: [rewrite.py#L354-L395](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L354-L395)
- [PT6] in-place `setattr`: [rewrite.py#L742-L753](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L742-L753)
- [PT7] assert text by tokenizing: [rewrite.py#L551-L552](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L551-L552), [rewrite.py#L906](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L906)
- [PT8] collection imports the module: [python.py#L501-L512](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/python.py#L501-L512), [python.py#L557-L561](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/python.py#L557-L561)
- [PT9] collection reads `__dict__`: [python.py#L400-L411](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/python.py#L400-L411)
- [PT10] a failure report parses again: [_code/source.py#L181-L193](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/_code/source.py#L181-L193)
- [PT11] the per-report `astcache`: [_code/code.py#L287-L309](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/_code/code.py#L287-L309), [_code/code.py#L896](https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/_code/code.py#L896)

Pyright:

- [PY1] `ParseNodeBase`, `id` and `parent`: [parseNodes.ts#L167-L187](https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/parser/parseNodes.ts#L167-L187)
- [PY2] `AnalyzerNodeInfo` in a `WeakMap`: [analyzerNodeInfo.ts#L40-L60](https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/analyzer/analyzerNodeInfo.ts#L40-L60), [analyzerNodeInfo.ts#L112-L113](https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/analyzer/analyzerNodeInfo.ts#L112-L113)
- [PY3] `dropParseAndBindInfo`: [sourceFile.ts#L474-L495](https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/analyzer/sourceFile.ts#L474-L495)
- [PY4] `getParseResults`: [sourceFile.ts#L648-L655](https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/analyzer/sourceFile.ts#L648-L655)
- [PY5] the background thread gets text: [backgroundAnalysisBase.ts#L133-L138](https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/backgroundAnalysisBase.ts#L133-L138)
- [PY6] `TextEditAction`: [editAction.ts#L13-L16](https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/common/editAction.ts#L13-L16)

libcst:

- [LC1] frozen nodes and `with_changes`: [base.py#L111](https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/_nodes/base.py#L111), [base.py#L303-L312](https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/_nodes/base.py#L303-L312)
- [LC2] `MetadataWrapper`: deep copy, identity, and metadata kept per provider: [wrapper.py#L109-L202](https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/metadata/wrapper.py#L109-L202)
- [LC4] each codemod worker parses its own file: [_cli.py#L294-L320](https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/codemod/_cli.py#L294-L320), [_cli.py#L633-L637](https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/codemod/_cli.py#L633-L637)

CPython:

- [CP1] `compile()` accepts "an AST object": [functions.html#compile](https://docs.python.org/3.14/builtins/functions.html#compile)

purba:

- [S1] the parts and their dependencies: [LANDSCAPE.md#L55-L80](https://github.com/kalonji-tools/purba/blob/499a8c8d777921b4e035ee35e4ff208dbc7e1bf7/LANDSCAPE.md#L55-L80)
- [S2] assertion compiles the rewritten source: [the-skeleton-crosses-the-seam-in-execution-and-assertion.md](https://github.com/kalonji-tools/purba/blob/499a8c8d777921b4e035ee35e4ff208dbc7e1bf7/docs/decisions/the-skeleton-crosses-the-seam-in-execution-and-assertion.md#decision-outcome)
- [S3] each part is a crate: [each-part-is-a-crate.md](https://github.com/kalonji-tools/purba/blob/499a8c8d777921b4e035ee35e4ff208dbc7e1bf7/docs/decisions/each-part-is-a-crate.md#decision-outcome)

[A1]: https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/node_index.rs.html#19-61
[A2]: https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/node_index.rs.html#156-180
[A3]: https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/generated.rs.html#9277-9284
[A4]: https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/generated.rs.html#9299-9311
[A5]: https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/generated.rs.html#9476-9483
[A6]: https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/generated.rs.html#5831
[A7]: https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/find_node.rs.html#7-14
[A8]: https://docs.rs/ruff_python_ast/0.0.17/src/ruff_python_ast/name.rs.html#36-38
[P1]: https://docs.rs/ruff_python_parser/0.0.17/src/ruff_python_parser/lib.rs.html#385-392
[P3]: https://docs.rs/ruff_python_parser/0.0.17/src/ruff_python_parser/parser/mod.rs.html#225-229
[D1]: https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#19-43
[D2]: https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#107-162
[D3]: https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#189-206
[D4]: https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#572-622
[D5]: https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#624-650
[D6]: https://docs.rs/ruff_db/0.0.17/src/ruff_db/parsed.rs.html#228
[T1]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_python_core/src/ast_node_ref.rs#L13-L56
[T2]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_python_core/src/ast_node_ref.rs#L74-L114
[T3]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_python_core/src/expression.rs#L111-L139
[T4]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_python_core/src/lib.rs#L74-L83
[T5]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ty_project/src/lib.rs#L545-L576
[L1]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/linter.rs#L121-L135
[L2]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/linter.rs#L446-L457
[L3]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/linter.rs#L560-L645
[L4]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/checkers/ast/mod.rs#L196-L234
[L5]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/checkers/ast/mod.rs#L3552-L3631
[L6]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_semantic/src/nodes.rs#L16-L87
[L7]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/checkers/ast/deferred.rs#L8-L16
[L8]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_linter/src/fix/mod.rs#L54-L121
[L9]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_diagnostics/src/edit.rs#L12-L17
[L10]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff/src/commands/check.rs#L83
[L11]: https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff/src/cache.rs#L335-L361
[RA1]: https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/syntax/src/ptr.rs#L1-L10
[RA2]: https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/syntax/src/ptr.rs#L21-L65
[RA3]: https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/syntax/src/lib.rs#L69-L98
[RA4]: https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/base-db/src/editioned_file_id.rs#L20-L28
[RA5]: https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-expand/src/lib.rs#L1752-L1757
[RA6]: https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/span/src/ast_id.rs#L1-L21
[RA7]: https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/span/src/ast_id.rs#L600-L609
[RA8]: https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/syntax/src/syntax_editor.rs#L30-L49
[RA9]: https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/ide-db/src/source_change.rs#L290-L292
[RA10]: https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/ide-db/src/text_edit.rs#L18-L29
[RA11]: https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/span/src/ast_id.rs#L44-L54
[RW1]: https://docs.rs/rowan/0.17.0/src/rowan/ast.rs.html#50-87
[PT1]: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/__init__.py#L129-L133
[PT2]: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L68-L70
[PT3]: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L148-L188
[PT4]: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L343-L351
[PT5]: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L354-L395
[PT6]: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L742-L753
[PT7]: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/assertion/rewrite.py#L551-L552
[PT8]: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/python.py#L501-L512
[PT9]: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/python.py#L400-L411
[PT10]: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/_code/source.py#L181-L193
[PT11]: https://github.com/pytest-dev/pytest/blob/cf470ec0bf7eb89cd97dd56df4859eae5db46447/src/_pytest/_code/code.py#L287-L309
[PY1]: https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/parser/parseNodes.ts#L167-L187
[PY2]: https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/analyzer/analyzerNodeInfo.ts#L40-L60
[PY3]: https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/analyzer/sourceFile.ts#L474-L495
[PY4]: https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/analyzer/sourceFile.ts#L648-L655
[PY5]: https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/backgroundAnalysisBase.ts#L133-L138
[PY6]: https://github.com/microsoft/pyright/blob/a28cce4077d87b80bead12de809f79845e39c8ba/packages/pyright-internal/src/common/editAction.ts#L13-L16
[LC1]: https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/_nodes/base.py#L303-L312
[LC2]: https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/metadata/wrapper.py#L109-L202
[LC4]: https://github.com/Instagram/LibCST/blob/c029c17bf45a3737fc8d1347001ab2422f42ae58/libcst/codemod/_cli.py#L294-L320
[CP1]: https://docs.python.org/3.14/builtins/functions.html#compile
[S1]: https://github.com/kalonji-tools/purba/blob/499a8c8d777921b4e035ee35e4ff208dbc7e1bf7/LANDSCAPE.md#L55-L80
[S2]: https://github.com/kalonji-tools/purba/blob/499a8c8d777921b4e035ee35e4ff208dbc7e1bf7/docs/decisions/the-skeleton-crosses-the-seam-in-execution-and-assertion.md#decision-outcome
[S3]: https://github.com/kalonji-tools/purba/blob/499a8c8d777921b4e035ee35e4ff208dbc7e1bf7/docs/decisions/each-part-is-a-crate.md#decision-outcome
