# #1230 prototype: one query language on pest and on chumsky, against oxitest's `-E`

This branch is never merged. Each purba cell below comes from a run of a wheel, and names its run file.
Each oxitest cell comes from oxitest's own parser and evaluator, built alone. The predictions were committed before the grammar existed, in [`PREDICTIONS.md`](PREDICTIONS.md).

pest and chumsky picked the same tests, and parsed the same files, in every row where the depth limit is on. They differ only in the text of a parse error, and in what each crate does with a deep query when the limit is off.
Against oxitest, purba's grammar differs in five ways that a tester sees. `&` binds tighter than `|`, so the precedence row picks 7 tests where oxitest picks 4. An id matches only at a boundary of the id, so a typo is an error that names the nearest id, where oxitest's substring match picked a test silently. A query that picks nothing exits 3, where oxitest exits 0. A query nested 256 levels deep is refused, where oxitest takes 10000 levels. Stage one parses only the files that an id or a `path()` can reach, where oxitest parses every file.
Five results went against my predictions, all with the depth limit off or outside the grammar. The largest: pest's own stack guard stopped 10000 levels before purba's walk could overflow, but purba's walk from pest's pairs overflowed at 3000. Each of the five is listed below the grid.

## Wheels and harnesses

| what | commit | how it ran |
|---|---|---|
| purba | `979ecff8` (`proto/1230-query`) | `mise run build` on a clean tree, a debug build; installed with `pip install --no-index --find-links target/wheels` into a scratch venv of CPython 3.12.14 |
| oxitest | [`731c2b3f`](https://github.com/kalonji-tools/oxitest/tree/731c2b3f/src/query) | [`oxitest/run.sh`](oxitest/run.sh) copies `ast.rs`, `compile.rs`, `eval.rs` and `resource.rs` unchanged and builds them with [`oxitest/main.rs`](oxitest/main.rs), a release build; the tests are [`oxitest/entries.tsv`](oxitest/entries.tsv) |

Linux 7.3.4, a stack of 8 MiB (`ulimit -s` 8192). The scenarios are in [`scenarios.sh`](scenarios.sh), the harness of purba in [`run.sh`](run.sh):

```sh
proto/1230/oxitest/run.sh <oxitest checkout> <empty dir> proto/1230/runs/oxitest
proto/1230/run.sh <venv>/bin/purba proto/1230/runs <work dir> <dir of the 9 suites>
```

`run.sh` copies [`suite/`](suite) to a scratch directory and runs each scenario there with `PURBA_PROTO_LIST=1`, which prints the picks and imports nothing. The suite holds 4 files and 10 tests. Each run file holds the command, the switches, stdout, stderr and the exit code.

## The grid

A purba cell reads: tests picked · files parsed of 4 · the error. "error" is an invocation error, exit 3. oxitest's "parsed" is 4 in every row: oxitest prescans every module, then tests `-E` on each one to decide which to import ([`prescanned.rs` L72-L83](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/pipeline/transitions/prescanned.rs#L72-L83), [`mod.rs` L107-L130](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/pipeline/transitions/mod.rs#L107-L130)) [source]. An oxitest cell that picks 0 exits 0, as oxitest does by design ([`exit-codes.md` L33](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/docs/user/reference/exit-codes.md#L33)).

The mark column: **P** where a purba result differs from its prediction; **O** where purba differs from oxitest.

| scenario | purba query | oxitest `-E` | pest | chumsky | prediction | mark |
|---|---|---|---|---|---|---|
| q1 slow | `tag(slow)` | 5 | 5 · 4 | 5 · 4 | 5 · 4 | |
| q2 slow or db | `tag(slow) \| tag(db)` | 7 | 7 · 4 | 7 · 4 | 7 · 4 | |
| q3 slow, not db | `tag(slow) & !tag(db)` | 4 | 4 · 4 | 4 · 4 | 4 · 4 | |
| q4 slow of `store` | `tests/store & tag(slow)` | 3 | 3 · 1 | 3 · 1 | 3 · 1 | O: parsed |
| q5 slow and db | `tag(slow) & tag(db)` | 1 | 1 · 4 | 1 · 4 | 1 · 4 | |
| q6 slow, or a file | `tag(slow) \| tests/api/test_admin.py` | 6 | 6 · 4 | 6 · 4 | 6 · 4 | |
| q7 not both | `!(tag(slow) & tag(db))` | 9 | 9 · 4 | 9 · 4 | 9 · 4 | |
| q8 either, not both | `(tag(slow) \| tag(db)) & !(…)` | 6 | 6 · 4 | 6 · 4 | 6 · 4 | |
| q9 no tag | `!tag(slow) & !tag(db) & !tag(flaky)` | 3, by `!mark()` | 3 · 4 | 3 · 4 | 3 · 4, by naming each tag | O: oxitest has "any mark" |
| precedence | `tag(db) \| tag(slow) & !tag(db)` | **4** | **7** · 4 | **7** · 4 | 7 · 4 | O |
| precedence, grouped | `tag(db) \| (tag(slow) & !tag(db))` | 7 | 7 · 4 | 7 · 4 | 7 · 4 | |
| id, a directory | `tests/api` | 3 | 3 · 2 | 3 · 2 | 3 · 2 | O: parsed |
| id, a file | `tests/api/test_user.py` | 2 | 2 · 1 | 2 · 1 | 2 · 1 | O: parsed |
| id, a test | `tests/api/test_user.py::test_add` | 1 | 1 · 1 | 1 · 1 | 1 · 1 | O: parsed |
| id, a case | `…/test_user.py::test_add::neg` | 0 | 0 · 1 · error, nearest `tests/api/test_user.py::test_add` | same | same | O: exit |
| typo, the issue's form | `store::migrat` | 0 | 0 · 0 · error, nearest **`tests/math`** | same | 0 · 0 · error, nearest a path | O: exit |
| typo in a test id | `tests/store/test_store.py::test_migrat` | **1**, `test_migrate` by substring | 0 · 1 · error, nearest `tests/store/test_store.py::test_migrate` | same | same | O |
| typo in a directory | `tests/stor` | **4**, by substring | 0 · 0 · error, nearest `tests/store` | same | same | O |
| name | `name(add)` | 3 | 3 · 4 | 3 · 4 | 3 · 4 | |
| name of a file | `name(math)` | **3**: `name` reads the whole node id | 0 · 4 · error, nearest name `test_add` | same | 0 · 4 · error | O |
| tag typo | `tag(slwo)` | 0 | 0 · 4 · error, nearest tag `slow` | same | same | O: exit |
| empty combination | `tag(flaky) & tests/api` | 0 | 0 · 2 · error: "`tag(flaky)` picks no test; the nearest tag is `slow`" | same | same, misleading as predicted | O: exit |
| unclosed `(` | `tag(slow) & (tag(db)` | error: "unmatched ')'" | error at `1:14`, "expected not" | error at column 21, "found end of input expected '&', '\|', or ')'" | pest near 14, chumsky 21 | O |
| missing operand | `tag(slow) &` | error: "empty expression" | error at `1:12`, "expected not, tag, name, path, word, or id" | error at column 12, "expected '!', 't', 'n', 'p', any, or '('" | 12 in both | |
| empty query | `''` | error: "empty expression" | error at `1:1`, "expected and" | error at column 1, "found end of input expected '!', 't', …" | 1 in both | |
| 255 `(` | `(((…tag(slow)…)))` | 5 | 5 · 4 | 5 · 4 | 5 · 4 | |
| 256 `(` | | **5** | error: "the query nests deeper than 255 levels at column 256" | same | same | O |
| 257 `(` | | 5 | the same error, column 256 | same | same | O |
| 10000 `(` | | 5 | the same error, column 256 | same | same | O |
| 255 `!` | `!!!…tag(slow)` | 5 | 5 · 4 | 5 · 4 | 5 · 4 | |
| 256 `!` | | 5 | the same error, column 256 | same | same | O |
| 3000 `(`, limit off | | not run | **crash, exit 139**, in purba's walk from pest's pairs | 5 · 4 | not predicted | P |
| 10000 `(`, limit off | | 5 | **error from pest: "stack limit reached" at `1:3061`** | 5 · 4 | pest: purba's walk overflows, exit 134; chumsky 5 | P |
| 60000 `(`, limit off | | not run | error from pest, "stack limit reached" at `1:3024` | 5 · 4 | (predicted for 100000) pest's error; chumsky 5 | |
| 3000 `!`, limit off | | not run | 5 · 4 | 5 · 4 | not predicted | |
| 10000 `!`, limit off | | 5 | error from pest, "stack limit reached" at `1:7194` | **crash, exit 139** | not predicted | |
| 60000 `!`, limit off | | not run | error from pest, "stack limit reached" at `1:7151` | **crash, exit 139** | (predicted for 100000) a crash, exit 134, unless pest stops first | P: exit 139, not 134 |
| 256 terms in `&` | `!tag(flaky) & … & tag(slow)` | 4 | 4 · 4 | 4 · 4 | 4 · 4 | |
| 256 terms in `\|` | `!tag(flaky) \| … \| tag(slow)` | 10 | 10 · 4 | 10 · 4 | 10 · 4 | |
| 2000 ids in `\|` | `<id> \| <id> \| …`, 76 KB | not run | 2 · 2 | 2 · 2 | not predicted | |
| 10000 ids in `\|` | about 390 KB | not run | **not run: the shell refused it, "Argument list too long", exit 126** | same | not predicted | |
| alias | `slow_store` | no aliases | 3 · 1 | 3 · 1 | 3 · 1 | |
| alias with an argument | `stable(slow_store)` | no aliases | 2 · 1 | 2 · 1 | 2 · 1 | |
| alias with a query | `stable(tag(slow))` | no aliases | 4 · 4 | 4 · 4 | 4 · 4 | |
| unknown alias, a word | `nosuch` | no aliases | 0 · 0 · error, read as an id, nearest `tests` | same | same | |
| unknown alias, a call | `unstable(tag(slow))` | no aliases | error: "`unstable(…)` names no alias with an argument in [tool.purba.queries]; the nearest is `stable(x)`" | same | same | |
| alias names itself | `loop` | no aliases | error: "the alias `loop` names itself" | same | same | |
| alias cycle | `ping` | no aliases | error: "the alias `ping` names itself through `pong`" | same | same | |
| path and tag | `path(tests/api/**) & tag(slow)` | 1 | 1 · **2** | 1 · 2 | 1 · 2 | O: parsed |
| path only | `path(tests/api/**)` | 3 | 3 · 2 | 3 · 2 | 3 · 2 | O: parsed |
| not path, and tag | `!path(tests/api/**) & tag(slow)` | 4 | 4 · 2 | 4 · 2 | 4 · 2 | O: parsed |
| tag or path | `tag(slow) \| path(tests/api/**)` | 7 | 7 · 4 | 7 · 4 | 7 · 4 | |
| rerun the failures | `…::test_remove \| …::test_load` | 2 | 2 · 2 | 2 · 2 | 2 · 2 | O: parsed |

Run files: `runs/<scenario>.<pest or chumsky>.txt`, and `runs/oxitest/<scenario>.txt`. For example, the precedence row is [`runs/precedence.pest.txt`](runs/precedence.pest.txt) beside [`runs/oxitest/precedence.txt`](runs/oxitest/precedence.txt).
oxitest's forms for an id are `source("…")` and `name(…)`, since `-E` has no id. Its 255 and 256 negations picked 5 tests each, but not the same 5: 255 `!` pick the tests that are not slow.

### The 9 suites

purba only parsed these; it imported nothing. Each pair of cells is the same for pest and chumsky. "picked" counts the tests the query names, refused ones included.

| suite | files found | `name(test)`: parsed, picked | `path(<first file>) & name(test)`: parsed, picked | prediction |
|---|---|---|---|---|
| attrs | 26 | 26, 683 | 1, 16 | 26 / 1 |
| click | 43 | 43, 619 | 1, 4 | 43 / 1 |
| dateutil | 17 | 17, 1133 | 1, 1 | 17 / 1 |
| loguru | 50 | 50, 720 | 1, 10 | 50 / 1 |
| pyflakes | 13 | 13, 564 | 1, 37 | 13 / 1 |
| requests | 9 | 9, 347 | 1, 1 | 9 / 1 |
| schedule | 1 | 1, 81 | 1, 81 | 1 / 1 |
| six | 1 | 1, 77 | 1, 77 | 1 / 1 |
| toolz | 29 | 29, 182 | 1, 1 | 29 / 1 |

Run files: `runs/suites/<suite>.<all or one>.<parser>.txt`; the commit of each suite is in [`runs/suites/commits.txt`](runs/suites/commits.txt). oxitest would parse every file found in both columns [source, above].

### The wheel, running the tests

| run | pest and chumsky, the same lines | prediction | mark |
|---|---|---|---|
| `purba tests` | 10 cases: 8 passed, 2 failed, **exit 1**; stderr prints `the failures as one query: tests/api/test_user.py::test_remove \| tests/store/test_store.py::test_load` | 8 passed, 2 failed, exit 4 | P: exit |
| that query, run again | 2 cases: 2 failed, exit 1; 2 files parsed | 2 run, exit 4 | P: exit |
| `purba tests/math/test_math.py` | 3 cases: 3 passed, exit 0; 1 file parsed | 3 passed | |
| `purba 'tag(slow) & !tag(flaky)'` | 4 cases: 3 passed, 1 failed, exit 1 | 3 pass, 1 fails | |
| `purba 'stable(slow_store)'` | 2 cases: 2 passed, exit 0; 1 file parsed | not predicted | |
| `purba store::migrat` | exit 3 before any test, nearest `tests/math` | exit 3 | |

Run files: `runs/wheel/<run>.<parser>.txt`. `diff` of each pest file against its chumsky file, with the parser's name masked, found no other line that differs.

## Where a result differed from its prediction

1. **10000 `(` with the limit off, on pest.** I predicted that pest parses and purba's recursive walk then overflows. pest's own guard refused the parse first, with "stack limit reached", and exit 3. The column of that error moves from run to run: the saved files give `1:3061` for 10000 and `1:3024` for 60000, and an earlier run of the same harness gave `1:3024` and `1:3063`. The guard measures the stack, not the depth, so its position is not stable.
2. **3000 `(` with the limit off, on pest.** Not predicted. pest parsed it below its guard, and purba's recursive walk from pest's pairs to the tree overflowed, exit 139. A throwaway unit test (not committed, with `RUST_MIN_STACK` at 8 MiB) printed a line after pest's parse returned, then overflowed in the walk.
3. **60000 `!` with the limit off, on chumsky.** I predicted exit 134. It is 139: the process died by SIGSEGV, as at 10000. A throwaway unit test (not committed, in a test thread of 2 MiB, not the wheel) showed chumsky returns the tree for 10000 `!`, and the overflow is in the prototype's own recursive alias walk, which runs after the parse.
4. **The exit code of a failed test.** I predicted 4. A failed test exits 1, which is `main`'s code for a failure ([`exit_code` L319-L320](https://github.com/kalonji-tools/purba/blob/3093871b/crates/reporting/src/lib.rs#L319-L320)); 4 is not a failure code.
5. **The nearest id of `store::migrat`.** I predicted a path, and it is one, but not the one a tester would expect: `tests/math`, at edit distance 10, where `tests/store`, `tests/api` and `tests` are at 11.

Every other cell matched its prediction. The rows of 2000 and 10000 ids, 3000 `(` and 3000 `!` were added during the run.

## Lines of each grammar

Every line counts, comments and blank lines too, at `979ecff8`.

| crate | grammar | walk to the tree | total |
|---|---|---|---|
| pest 2.9.7 | [`query.pest`](../../crates/selection/src/query/query.pest): 17 | [`with_pest.rs`](../../crates/selection/src/query/with_pest.rs): 36 | 53 |
| chumsky 0.14.2 | [`with_chumsky.rs`](../../crates/selection/src/query/with_chumsky.rs): 62; the combinators build the tree as they parse | | 62 |

Both share [`query/mod.rs`](../../crates/selection/src/query/mod.rs): 496 lines before its tests, for the tree, the scan of depth, the aliases, stage one's verdict, a test's pick and the nearest name. That shared part is 8 to 9 times either grammar. In `git diff --stat origin/main`, stage one's discovery, the aliases' file and the run of several modules change 438 lines of `crates/command/src/lib.rs`, and reading `@purba.tag` changes 77 of `crates/selection/src/lib.rs`.

The two crates bring these to `Cargo.lock`: pest with `pest_derive`, `pest_generator`, `pest_meta`, `pest_stack_guard` and `ucd-trie`; chumsky with `hashbrown`, `foldhash`, `allocator-api2`, `regex-automata` and `unicode-segmentation`. `toml` came for the aliases. Each is MIT or Apache-2.0, except `foldhash` 0.2.0, which is Zlib. None is AGPL.

## The depth limit: 255

A query nests at most 255 levels. Each `(` that opens a group, each `!`, and each alias call opens one level. A scan of the text refuses a deeper query before either crate parses, and a check of the tree after the aliases expand refuses a tree deeper than 256 nodes.

I chose 255 for three reasons.

- The issue's rows 255 and 256 put the edge there, and jj's revsets refuse past 256, which [#1228](https://github.com/kalonji-tools/purba/issues/1228) measured.
- With the limit off, the shallowest failure measured is 3000 `(` on pest, where purba's walk overflows; pest's guard stops near column 3024; chumsky fails in purba's alias walk at 10000 `!`. All on this debug build. 255 is under the shallowest by a factor of more than 10.
- A list of ids is width, not depth. `&` and `|` build one node with a list of terms, so 2000 ids parsed and picked with the limit on.

The scan is shared, so the tester sees one text from both crates: "the query nests deeper than 255 levels at column 256".

## Provisional choices

The prototype made each of these choices. None of them is decided.

- `purba <query>` takes exactly one argument. The root is the working directory. A bare path such as `tests/math/test_math.py` is a query.
- Stage one lists `test_*.py` and `*_test.py` under the root, sorted by path, and skips a directory whose name starts with `.`, and `__pycache__`.
- An id with no `::` is a path prefix that ends at a `/`. An id with `::` matches a test whose id equals it or continues with `::`. So `tests/stor` and `…::test_migrat` match nothing, where oxitest's substring matched.
- `name(x)` matches a substring of the test's function's name, not its path.
- A literal in `tag()`, `name()` and `path()` takes letters, digits and `_-.:/[]`, and `path()` also `*` and `?`. No quotes.
- A bare word that names an alias is the alias, before it is an id. An alias call is `word(query)`; its key in `pyproject.toml` is quoted: `"stable(x)" = "x & !tag(flaky)"`.
- The error of an empty pick names each atom that picks nothing alone, and the nearest of its kind by edit distance, among what purba read. It reads only the files that stage one kept. So `tag(flaky) & tests/api` says `tag(flaky)` picks no test, though `test_load` carries it in a file stage one skipped.
- `@purba.tag("x")` resolves through the module's imports, the way selection's lookup resolves an imported name; `from purba import tag` counts. Selection no longer refuses it as a foreign decorator. At run time it returns the test unchanged.
- A refused test that the query picks still prints its refusal line.
- After a run with a failure, stderr prints the failures joined by ` | `.

## What I did not vary

- One build: the debug profile that `mise run build` gives. The depth at which each crate fails with the limit off depends on it, and on the 8 MiB stack. A release build is unmeasured.
- One Python (3.12.14), one OS (Linux), one machine.
- No error-message work in either crate: no renamed rules in pest, no labels in chumsky, no recovery. The parse errors are each crate's floor.
- Neither crate's Pratt helper: precedence comes from the levels of the rules in both.
- One discovery rule, one alias rule, one nearest-name metric (edit distance; the first on a tie).
- No class in the suite, and no case: purba has no parametrized case, so `…::test_add::neg` can only miss.
- No timing. I measured no speed of either crate.
- oxitest ran as its parser and evaluator over a list of entries, not as the oxitest binary. Its "parsed" column is [source].
- The 9 suites ran two queries each, in list mode only. No test of theirs was imported or run.
- A query of 10000 ids did not reach purba: Linux refuses one argument longer than 128 KiB. A long list of failures needs another way in than one argument (judgement).
