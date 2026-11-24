# #1230 prototype: what each scenario will do, written before any run

This branch is never merged.
This file is committed before the grammar exists in the crates, and before any run of it.
`RESULTS.md` sets each result beside its prediction here.

## The design the predictions assume

- `purba <query>` takes one argument, the query. The root is the working directory.
- Stage one lists every `test_*.py` and `*_test.py` under the root, and skips a directory whose name starts with `.`, and `__pycache__`.
  It decides each file yes, no or unknown from the parts of the query that read only a path: an id or a prefix of one, and `path()`.
  `tag()` and `name()` are unknown. `!`, `&` and `|` follow three-valued logic. A file is parsed unless it is no.
- An id with no `::` is a path prefix: a file matches when its path equals it or starts with it and `/`.
  An id with `::` matches a test whose id equals it or starts with it and `::`. In stage one, its part before the first `::` must equal the file's path.
- `tag(x)` matches a test that carries `@purba.tag("x")`, read from the AST. `purba.tag` resolves through the module's imports, so `from purba import tag` and `@tag("x")` count too.
- `name(x)` matches a test whose function's name holds `x`.
- `path(glob)` matches the file's path; `*` and `?` stay inside one directory, and `**` crosses directories.
- An alias is a bare word or a call `word(query)` that `[tool.purba.queries]` names. A bare word that names no alias is an id.
- The depth limit is 255 levels. Each `(` that opens a group, each `!`, and each alias call opens a level. A scan of the text counts them before either crate parses, and the tree is checked again after the aliases expand.
- `&` and `|` build one node with a list of terms, so width adds no depth.
- A query that selects no test is an invocation error with exit 3. It names each part that picks no test alone, and the nearest of its kind among what purba read: a path or a test id for an id, a tag for a tag, a function name for a name.
- `PURBA_PROTO_PARSER` picks `pest` or `chumsky`. `PURBA_PROTO_DEPTH=off` removes the limit, to show what each crate does without it. `PURBA_PROTO_LIST=1` prints the picks and imports nothing.

## The small suite

`suite/` holds 4 files and 10 tests. The tags are: `slow` on `test_add_slow`, `test_migrate`, `test_load`, `test_save` and `test_remove`; `db` on `test_query`, `test_migrate` and `test_grant`; `flaky` on `test_load`.
`test_load` and `test_remove` fail.

## Predictions

pest and chumsky build one tree, so I predict one result for both, except where a cell names a crate.
"parsed" counts files of 4. "error" is an invocation error, exit 3.

| scenario | picked | parsed | error text |
|---|---|---|---|
| q1-one-pick | 5 | 4 | none |
| q2-union | 7 | 4 | none |
| q3-exclusion | 4 | 4 | none |
| q4-id-and-tag | 3 | 1 | none |
| q5-intersection | 1 | 4 | none |
| q6-tag-or-id | 6 | 4 | none |
| q7-not-both | 9 | 4 | none |
| q8-either-not-both | 6 | 4 | none |
| q9-no-tag | 3, by naming each tag; the grammar has no "any tag" | 4 | none |
| precedence | 7; oxitest picked 4 | 4 | none |
| precedence-grouped | 7 | 4 | none |
| id-dir | 3 | 2 | none |
| id-file | 2 | 1 | none |
| id-test | 1 | 1 | none |
| id-case | 0 | 1 | error; nearest `tests/api/test_user.py::test_add` |
| id-typo-issue `store::migrat` | 0 | 0 | error; the nearest is a path, not a test id, since no file is parsed |
| id-typo-test | 0 | 1 | error; nearest `tests/store/test_store.py::test_migrate` |
| id-typo-dir | 0 | 0 | error; nearest `tests/store` |
| name-add | 3 | 4 | none |
| name-math | 0 | 4 | error; names a function name |
| tag-typo | 0 | 4 | error; nearest tag `slow` |
| empty-combination | 0 | 2 | error; it says `tag(flaky)` picks no test, though `test_load` carries it, because stage one did not parse its file |
| unclosed | 0 | 0 | error; pest points inside the group, near column 14; chumsky at the end, column 21, "found end of input" |
| missing-operand | 0 | 0 | error at column 12 in both |
| empty-query | 0 | 0 | error at column 1 in both |
| parens-255 | 5 | 4 | none |
| parens-256, parens-257, parens-10000 | 0 | 0 | error from the scan, the same text in both crates, at column 256 |
| bangs-255 | 5, the tests that are not slow | 4 | none |
| bangs-256 | 0 | 0 | error from the scan |
| parens-10000, limit off | pest: the tree walk overflows the stack, exit 134; chumsky: 5 | chumsky 4 | none |
| parens-100000, limit off | pest: its "stack limit reached" error; chumsky: 5 | chumsky 4 | pest's text |
| bangs-100000, limit off | both crash on the deep tree, exit 134, unless pest stops first | | |
| width-256-and | 4 | 4 | none |
| width-256-or | 10 | 4 | none |
| alias `slow_store` | 3 | 1 | none |
| alias-with-argument `stable(slow_store)` | 2 | 1 | none |
| alias-with-query `stable(tag(slow))` | 4 | 4 | none |
| alias-unknown-word `nosuch` | 0 | 0 | error; read as an id; nearest is a path |
| alias-unknown-call `unstable(tag(slow))` | 0 | 0 | error; no alias `unstable`; nearest alias `stable` |
| alias-names-itself `loop` | 0 | 0 | error; `loop` names itself |
| alias-cycle `ping` | 0 | 0 | error; `ping` names itself through `pong` |
| path-and-tag | 1 | 2 | none |
| path-only | 3 | 2 | none |
| not-path-and-tag | 4 | 2 | none |
| tag-or-path | 7 | 4 | none |
| rerun-failures | 2 | 2 | none |

oxitest parses every file before it tests `-E` on it, so its "parsed" is 4 in every row (`transitions/mod.rs` L107-L126 at `731c2b3f`).

## The 9 suites

| suite | files found | `name(test)` parses | `path(<its first file>) & name(test)` parses |
|---|---|---|---|
| attrs | 26 | 26 | 1 |
| click | 43 | 43 | 1 |
| dateutil | 17 | 17 | 1 |
| loguru | 50 | 50 | 1 |
| pyflakes | 13 | 13 | 1 |
| requests | 9 | 9 | 1 |
| schedule | 1 | 1 | 1 |
| six | 1 | 1 | 1 |
| toolz | 29 | 29 | 1 |

## The wheel

- `purba tests` runs 10 tests: 8 pass, 2 fail, exit 4. It prints the failures as one query.
- That query, run again, runs the 2 tests, which fail again, exit 4.
- `purba tests/math/test_math.py` runs 3 tests and passes, as `purba <file>` does today.
- `purba 'tag(slow) & !tag(flaky)'` runs 4 tests: 3 pass, 1 fails.
- `purba store::migrat` exits 3 before any test runs.
- pest and chumsky give the same lines in each of these runs.
