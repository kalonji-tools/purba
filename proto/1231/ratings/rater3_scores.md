# Rater 3 (PORT_R = oxitest, PORT_T = purba)

The rater's final report, word for word, with its opening label dropped, the absolute path of its `_review/` folder shortened, and its section titles set as headings. The rater named each port's runner from its folder.

I ran every criterion myself; all scratch work is in `_review/`.

**Summary.** PORT_R is the oxitest port and PORT_T is the purba port. PORT_R comes first on my weighting. Its tests state their intent most clearly: named cases, an assert message on every check, and clean type checkers. It also re-runs failures best. Its cost is the most to learn: strict rules, dataclass cases, and two workarounds. The original pytest version is the shortest and easiest to edit, but a typo in a test name goes completely silent. PORT_T is the flattest code and gives the best hints for a bad query. It loses points because re-running failures means copying a 107-character query, a mistyped tag is silent, and pyright reports `purba.tag` as unknown.

## 1. Scores (5 = best; for both complexity rows, 5 = simplest)

| Criterion | ORIGINAL | PORT_R (oxitest) | PORT_T (purba) |
|---|---|---|---|
| 1 Readability | 4 | 3 | 4 |
| 2 Clarity | 3 | 5 | 4 |
| 3 Ease of use | 4 | 4 | 3 |
| 4 Complexity for the tester | 3 | 2 | 3 |
| 5 Complexity of the implementation | 2 | 2 | 3 |
| 6 Minimum working test | 5 | 4 | 5 |
| 7 Fidelity | (ref) 5 | 5 | 4 |
| 8 Tagging | 4 | 3 | 3 |
| 9 Misspelled test name | 1 | 4 | 4 |
| 10 Re-running failures | 4 | 5 | 3 |

## 2. Per version

ORIGINAL (pytest)

- Strongest: compact. It has 111 non-blank lines, against 193 for PORT_R and 173 for PORT_T.
  - A new case is one tuple line.
  - A malformed 2-tuple gives a clear collection error: "the number of names (3) … must be equal to the number of values (2)".
- Weakest: `writer` and `reset_logger` arrive by name from conftest, so a newcomer cannot see where they come from.
  - ty misses the malformed tuple.
  - pyright already reports 11 errors (9 in conftest), so a new error is easy to miss.

PORT_R (oxitest)

- Strongest: each case has a name (`no_fields=FormatCase(...)`), and every assert has a message.
  - The failure shows the case name and the message: `└ format 'Nope' should render 'b' as 'Nop\n'`.
  - ty flags a wrong field type statically: "Expected `str`, found `Literal[7]`".
  - ty is clean.
- Weakest: the most machinery.
  - The `TYPE_CHECKING`/`else` block at test_add_option_format.py:15-23 is a workaround.
  - A 132-character suppression comment sits at :137.
  - Strict mode refuses any module-level helper: I added `def double()` and got "module-level-def double line 1 … aborting (exit 3)".
  - A teardown exception gives only "21 warnings" and exit 0. I probed this; it confirms the NOTES.

PORT_T (purba)

- Strongest: plain functions with no decorators and no fixtures. The failure output is direct: `expected 'Nop\n' / reading 'Nope\n'`.
  - A typo in a tag query gets a hint: `tag(smok)` printed "the nearest tag is `smoke`" and exited 3.
- Weakest: there is no parametrize, so it has 20 near-duplicate functions. For example, the 5 `test_invalid_markups_*` bodies at :151-178 are identical except for one string.
  - pyright gives 22 errors, among them "`tag` is not a known attribute of module purba" and `writer.read` on each `make_writer()` use.
  - The rules are strict, and the original dateutil test is refused with `[not-one-comparison] line 30`.

## 3. Criteria 8, 9 and 10, one line each

Criterion 8, tagging:

- ORIGINAL 4: a decorator typo printed "PytestUnknownMarkWarning: Unknown pytest.mark.smok - is this a typo?". A query typo (`-m smok`) gave "21 deselected", exit 5, with no hint.
- PORT_R 3: `-E 'mark(smok)'` silently ran the 2 smoke tests, because `mark()` matches substrings (`mark(sm)` does too). `mark(xyz)` gave "no tests ran", exit 0. A decorator typo was caught only on a full run: "unknown marker 'smok' … Hint: register it", exit 3.
- PORT_T 3: the query typo got the "nearest tag is `smoke`" hint (exit 3). But `@purba.tag("smok")` is silent, and forgetting `import purba` made `tag(smoke)` silently skip the test with exit 0.

Criterion 9, misspelled test name:

- ORIGINAL 1: pytest printed nothing; the only sign is "20 passed" instead of 21.
- PORT_R 4: strict mode aborted the whole run: "tset_progressive_format module-level-def … line 76 … aborting (exit 3)". It cannot be missed, but the message does not say "typo".
- PORT_T 4: it printed "purba: name-near: tests/test_add_option_format.py:88: tset_progressive_format is not a test, and its name is near the prefix test". The exit code is 0, so CI still passes.

Criterion 10, re-running failures:

- ORIGINAL 4: `pytest --lf` works. After I fixed the parametrized case, its id changed, so the next `--lf` said "rerun previous 1 failure" and never confirmed the fix.
- PORT_R 5: `oxitest --lf` keeps the case id stable (`no_fields`). After the fix it showed "1 failed · 1 passed", then ran only the remaining failure.
- PORT_T 3: purba keeps no record. I had to copy a 107-character query from the last line and quote it. Without quotes the shell printed "no such file or directory" and "Broken pipe". Each run prints a fresh query.

## 4. Overall ranking

Weights out of 100: readability 10, clarity 15, ease of use 15, tester complexity 10, implementation complexity 5, minimum test 5, fidelity 15, tagging 8, misspelled name 7, re-runs 10.

1. PORT_R: 3.92
2. ORIGINAL: 3.64 (scored 5 on fidelity because it is the reference)
3. PORT_T: 3.57

Without fidelity (renormalised to 85), the order is PORT_R 3.73, PORT_T 3.49, ORIGINAL 3.40.

## 5. Minimum working test

The files are `_review/min_ORIGINAL/`, `min_PORT_R/` and `min_PORT_T/`, each under `tests/test_min.py`; all pass. The body is the same in each:

```python
out = []
logger.add(out.append, format="{message}")
logger.debug("a")
assert out == ["a\n"]
```

- ORIGINAL: 5 non-blank lines, 1 import. Names to know: `logger`, the `test_` prefix.
- PORT_R: 5 lines, 1 import, and the assert must carry a message. Without one it gave "bare-assert line 8 … aborting (exit 3)". Names: `logger`, `test_`, plus the strict rule.
- PORT_T: 5 lines, 1 import. Names: `logger`, `test_`. purba does not capture output, so loguru's default stderr line showed up in the run.

## Fidelity detail (criterion 7)

- PORT_R checks the same as the original.
  - It checks more: assert messages, and typed cases.
  - It checks less: a teardown error no longer fails a test.
  - It has a `Writer` class without `.clear`, which no test used.
- PORT_T keeps every input and expected value.
  - The dateutil test is split into 4 tests that replay setup. This reports more, since each state passes or fails on its own.
  - `assert ymd.has_month` became `is True`, which is slightly stricter.
  - The teardown reset is gone.
  - `pytest.raises` became the `add_error` helper plus `isinstance` and `re.search`. It is equivalent: "DID NOT RAISE" now shows as `reading None (NoneType)`.

## 6. Bugs and rough edges (not scored)

- oxitest `mark()` matches substrings, and a query that selects nothing exits 0 with "no tests ran".
- oxitest reports an unknown marker only on unfiltered runs.
- oxitest's "DID NOT RAISE" failure points first at its own `_bridge/_raises.py:40`.
- `purba.tag` exists at runtime, but ty and pyright both report it as unknown.
- With `import purba` missing, a full purba run gives a misleading "[foreign-decorator] the decorator purba.tag is not purba's" next to the real NameError.
- pytest `--lf` loses a fixed parametrized case whose id comes from its values.
- PORT_R's NOTES say "ty and pyright are clean", but pyright reports 2 errors in test_internals.py. They come from the `_ymd` string argument, which the original has too.
- PORT_T's NOTES items 1-4 say `[computed-expected]`, `[second-path]`, the private-name warning and "blocked" are undocumented. The current rules document all four (TESTS.md:25, :29, :42, :40), so those notes are stale. Item 5 is confirmed.
