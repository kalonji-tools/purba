# The prompt each rater got

The prompt of the loguru round of the 1015 rating ([`RATER_PROMPT_loguru.md` at `107b09a6`](https://github.com/kalonji-tools/purba/blob/107b09a66bcdc7a4a8cba9cdd421beb96608e287/prototypes/1015-form/benchmark/ratings/RATER_PROMPT_loguru.md)), with these changes:

- The opening names two files, dateutil's test beside loguru's, and three runners where it named two forms.
- The folder list says that each version holds both files as one project, and that the rules of one port are in `rules/`, where it said `FORM.md`.
- The tools name purba and python-dateutil in the venv, and allow each runner's own cache in its folder, since `--lf` needs one.
- Each change is made in a copy under `_review/`.
- Criteria 8, 9 and 10 are new: the three forms. Criteria 1 to 7 are word for word.
- The report's table has ten criteria, and asks for each form's score apart.

`<eval>` is the eval folder; raters 2 and 3 got `rater2/` and `rater3/`.

```text
You are one of three independent reviewers. Two real pytest test files, loguru's `test_add_option_format.py` (9 tests, 20 items) and dateutil's `test_internals.py` cut to its one test `test_YMD_could_be_day` (marked `smoke`), have been ported to two other Python test runners. Score all three versions. Nobody has told you which version anyone prefers; don't guess, judge only on the evidence in your folder.

**Your folder (the only place you may read):** `<eval>/blind/rater1/`
- `ORIGINAL_pytest/` — the original files and the two fixtures they use from loguru's conftest.
- `PORT_R/` and `PORT_T/` — the two ports. Each version is one project that holds both files under `tests/`. Each port has its own `RUN.md`, `OUTPUT.txt`, `CHECKERS.txt`, and a `NOTES.md` written by the person who ported it (treat the notes as their claims: verify what matters). One port is in a runner with its own rules sheets (`rules/`).
Do not read other paths, do not use git or the web. You may write scratch files only under `<your folder>/_review/`. To change a version, copy its folder into `_review/` and change the copy. A runner may write its own cache in the folder it runs in.

**Tools:** each folder's `RUN.md` gives the exact run command (a shared venv with pytest 9.2.0, oxitest 4.0.0, purba, loguru 0.8.0 and python-dateutil is at `<eval>/.venv/`; do not install anything). `ty` is on PATH (`ty check --python <venv>/bin/python …`); pyright via `nix-shell -p pyright --run 'pyright …'` using each folder's `pyrightconfig.json`. Run things; do not rate from reading alone.

**Criteria — score each version 1–5 (5 = best; for both complexity rows, 5 = simplest), each score with concrete evidence (a `file:line`, a quoted line, a measured number, or real output):**
1. **Readability** — how easily a Python developer new to the form scans the file and follows it.
2. **Clarity** — how unambiguous each test's intent is from the code alone: what goes in, what is checked, what is expected, and what a failure on screen means.
3. **Ease of use** — writing and changing tests: do this for real in each version — add one new case to the `test_format` cases (message "g", format "{message}!", expected "g!\n"), then add one new test of your own choosing — and report how long/awkward each was, what the type checkers caught, and how helpful the framework's own errors were.
4. **Complexity for the tester** — concepts, rules and special names a tester must learn.
5. **Complexity of the implementation** — what the form/framework itself must be to support this (for the two ports: read their framework or runner code where it is in the folder; for a published framework, judge from what its API demands and what the port had to work around).
6. **Minimum working test** — write, in each version, the smallest passing test that checks that `logger.add(writer, format="{message}")` followed by `logger.debug("a")` writes `"a\n"`. Save each under `_review/`, run it, and report non-blank lines, imports, and names the tester must know.
7. **Fidelity** (ports only; the original is the reference) — does the port still test what each original test tests? List what it checks less, more, or differently.
8. **Tagging a test** — in each version, put the tag (mark) `smoke` on one more test, `test_progressive_format`, then run only the tests that carry `smoke`, then run it once with the tag misspelled as `smok`. Report what you had to write and declare, which tests ran, and what the runner said each time.
9. **A misspelled test name** — in each version, rename `test_progressive_format` to `tset_progressive_format`, as a typo would, and run the whole suite. Report what the runner printed about it, and how a tester would learn that this test no longer runs.
10. **Running the failures again** — in each version, seed failures: change the expected value in two different tests so that they fail, and run the suite. Then run only the failures again, as that version lets you, fix one, and run the failures again. Report each command you typed, what each run printed, and how much you had to copy, remember or retype.

**Report** (your final answer, plain text, under ~1200 words):
1. A table: the three versions × the ten criteria.
2. Per version, 3–5 lines: strongest and weakest point with evidence.
3. Criteria 8, 9 and 10 again, one line per version each: the score and the one piece of evidence that decided it.
4. Your overall ranking with the weighting stated explicitly.
5. The minimum-working-test counts.
6. Any bug or rough edge you hit (not scored).
```
