# The blind rating of selection's three forms. Never merge.

Three raters, each in a fresh context, scored three forms of purba beside pytest and oxitest: a tag on a test, the notice of a name near the prefix `test`, and the failures of a run printed as one query.
Each rater ran each version, ported from loguru's `test_add_option_format.py` and dateutil's `test_YMD_could_be_day`, and did each task of [the prompt](ratings/RATER_PROMPT.md).

**A round compares only with itself.**

## The three forms

5 is best. Each cell is the mean of the three raters, then each rater's score.

| form | the task | pytest | oxitest | purba |
|---|---|--:|--:|--:|
| a tag on a test | tag `test_progressive_format`, select by it, misspell it in the query | 3.67 (3 · 4 · 4) | 2.33 (2 · 2 · 3) | **3.67** (4 · 4 · 3) |
| a name near `test` | rename the test `tset_progressive_format`, run all | 1.00 (1 · 1 · 1) | 4.00 (4 · 4 · 4) | **4.67** (5 · 5 · 4) |
| the failures as one query | seed two failures, rerun them, fix one, rerun | 4.00 (4 · 4 · 4) | 5.00 (5 · 5 · 5) | **3.00** (3 · 3 · 3) |

What decided each score, as the raters wrote it. A ✓ marks a claim that a fresh probe confirmed ([`ratings/redteam.txt`](ratings/redteam.txt), [`ratings/redteam2.txt`](ratings/redteam2.txt)).

- **The tag.** purba's `tag(smok)` exits 3 and names the nearest tag `smoke` (3 of 3). oxitest's `-E 'mark(smok)'` matches a substring and runs the smoke tests, exit 0 (3 of 3) ✓. pytest's `-m smok` deselects all, exit 5, with no hint (3 of 3). Against purba: a misspelled tag in the decorator is silent (3 of 3). A tag in a file that does not `import purba` is not read, and `tag(smoke)` skips that test with exit 0 (raters 1 and 3) ✓. Both type checkers report `purba.tag` as unknown (3 of 3).
- **The name.** purba prints `purba: name-near: tests/test_add_option_format.py:88: tset_progressive_format is not a test, and its name is near the prefix test` and runs the rest, exit 0 (3 of 3) ✓. oxitest's strict mode aborts the run, `module-level-def tset_progressive_format line 76`, exit 3 ✓; the raters counted "cannot be missed" for it, and "does not say typo" against it. pytest says nothing; only the count drops (3 of 3). Against purba: exit 0, so a CI run passes (raters 2 and 3).
- **The rerun.** oxitest's and pytest's `--lf` need no copy (3 of 3). pytest's `--lf` loses a parametrized case whose id holds the expected value, so a fix is never rerun (3 of 3) ✓. purba's query is 105 to 120 characters to copy and quote, and nothing is kept between runs (3 of 3); unquoted, the shell refused it (raters 1 and 3).

## The rest of the round

The seven criteria of the 1015 rating, word for word, as context. For both complexity rows, 5 is simplest.

| criterion | pytest | oxitest | purba |
|---|--:|--:|--:|
| readability | 4.00 | 3.00 | 4.00 |
| clarity | 3.00 | 5.00 | 3.67 |
| ease of use | 4.67 | 3.67 | 3.00 |
| complexity for the tester | 3.00 | 2.00 | 3.33 |
| complexity of the implementation | 2.00 | 2.00 | 3.33 |
| smallest test | 5.00 | 3.67 | 4.67 |
| fidelity | reference | 5.00 | 4.00 |

Each rater's own ranking, by its own weighting: rater 1 purba, then pytest and oxitest tied; rater 2 purba 3.74, pytest 3.71, oxitest 3.68; rater 3 oxitest 3.92, pytest 3.64, purba 3.57.
Against purba's port, all three raters counted one test for each case, since purba has no parametrize and no fixture yet. Rater 1 also counted a "does not raise" check that blocks with `[second-path]`, exit 4, where pytest reports a failure ✓; rater 3's mutant of another test failed with `reading None`.

## Faults of this wheel and this kit, not of the forms

- The wheel ships no type stub, so ty and pyright report `purba.tag` as unknown. Raters 2 and 3 named it against the tag.
- A tag with no `import purba` gives `[foreign-decorator] the decorator purba.tag is not purba's`, a wrong cause, on a full run (raters 1 and 3).
- `purba.tag(SLOW)`, a name in place of a literal, gives the same wrong cause (a probe before the port; no rater met it).
- The port's `NOTES.md` lists five gaps of the rules that the porter met. They were fixed in [`rules/TESTS.md`](rules/TESTS.md) before the raters read it, so items 1 to 4 read stale (rater 3).
- purba does not capture output, so loguru's default line printed in each smallest test (3 of 3).

## How it ran

| what | where |
|---|---|
| purba | `d6cec0ba846378ad84f505c1a99a943ae9e9d04a` on this branch: the query of `979ecff80f27b0322e3424e4fee5f0154adf37d3` and the name-near check of `a5a7b5309e77b39da5d5639a75cf3f591dca5bc9`, without its `asserts` and `name-prefix` lines; `mise run build` on a clean detached checkout, a debug build |
| the venv | CPython 3.12.15, pytest 9.2.0, oxitest 4.0.0, loguru 0.8.0, python-dateutil at `a78cbd81` |
| purba's rules | [`rules/`](rules/): `TESTS.md` (not rated), `QUERY.md` (not rated), `TAG.md`, `NAME-NEAR.md`, `RERUN.md` |
| the port | [`benchmark/purba/`](benchmark/purba), by a fresh context that read only `rules/` and the originals ([`ratings/PORTER_PROMPT.md`](ratings/PORTER_PROMPT.md)); 24 cases pass, `tag(smoke)` picks 4 |
| pytest and oxitest | [`benchmark/pytest/`](benchmark/pytest), [`benchmark/oxitest/`](benchmark/oxitest); loguru's files unchanged from the 1015 rating, dateutil's test ported by hand for oxitest |
| the raters | [`ratings/RATER_PROMPT.md`](ratings/RATER_PROMPT.md), [`ratings/KEY.txt`](ratings/KEY.txt), [`ratings/build_blind.sh`](ratings/build_blind.sh); scores in [`ratings/rater1_scores.md.gz`](ratings/rater1_scores.md.gz) (gzipped, since it quotes a misspelling that the typos hook rewrites), [`rater2_scores.md`](ratings/rater2_scores.md), [`rater3_scores.md`](ratings/rater3_scores.md) |

The port of dateutil takes the whole test, lines 16 to 40 at `a78cbd81`. purba's port splits it into four tests, one for each state, since an assert before an act blocks a case.
