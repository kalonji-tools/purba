# The prompt each rater got

The prompt of the rating of selection's three forms ([`RATER_PROMPT.md` at `6266c8a1`](https://github.com/kalonji-tools/purba/blob/6266c8a1/proto/1231/ratings/RATER_PROMPT.md)), with these changes:

- The opening names one file, loguru's, and four versions where it named three: two of the ports run on purba, with other words in its output.
- The folder list names three ports, and says that each purba port has its rules in `rules/`.
- Each version has its own venv path in its `RUN.md`; the shared venv holds no python-dateutil.
- Criteria 1 to 7 are word for word. Criteria 8 and 9 are new: the two forms. Criterion 10 seeds failures, which the steps of a rating ask for.
- The report's table has four versions.

`<eval>` is the eval folder, and `<n>` the rater's number.

```text
You are one of three independent reviewers. A real pytest test file, loguru's `test_add_option_format.py` (9 tests, 20 items), has been ported to two other Python test runners; one of them was tried in two builds that print different words. Score all four versions. Nobody has told you which version anyone prefers; don't guess, judge only on the evidence in your folder.

**Your folder (the only place you may read):** `<eval>/blind/rater<n>/`
- `ORIGINAL_pytest/` — the original file and the two fixtures it uses from loguru's conftest.
- `PORT_R/`, `PORT_S/` and `PORT_T/` — the three ports. Each version is one project with its file under `tests/` and a `pyproject.toml`. Each port has its own `RUN.md`, `OUTPUT.txt`, `CHECKERS.txt`, and a `NOTES.md` written by the person who ported it (treat the notes as their claims: verify what matters). A port in a runner with its own rules sheets has them in `rules/`.
Do not read other paths, do not use git or the web. You may write scratch files only under `<your folder>/_review/`. To change a version, copy its folder into `_review/` and change the copy. A runner may write its own cache in the folder it runs in.

**Tools:** each folder's `RUN.md` gives the exact run command and the venv it uses (each venv holds pytest 9.2.0, oxitest 4.0.0, purba and loguru 0.8.0; do not install anything). `ty` is on PATH (`ty check --python <venv>/bin/python …`); pyright via `nix-shell -p pyright --run 'pyright …'` using each folder's `pyrightconfig.json`. Run things; do not rate from reading alone.

**Criteria — score each version 1–5 (5 = best; for both complexity rows, 5 = simplest), each score with concrete evidence (a `file:line`, a quoted line, a measured number, or real output):**
1. **Readability** — how easily a Python developer new to the form scans the file and follows it.
2. **Clarity** — how unambiguous each test's intent is from the code alone: what goes in, what is checked, what is expected, and what a failure on screen means.
3. **Ease of use** — writing and changing tests: do this for real in each version — add one new case to the `test_format` cases (message "g", format "{message}!", expected "g!\n"), then add one new test of your own choosing — and report how long/awkward each was, what the type checkers caught, and how helpful the framework's own errors were.
4. **Complexity for the tester** — concepts, rules and special names a tester must learn.
5. **Complexity of the implementation** — what the form/framework itself must be to support this (for the two ports: read their framework or runner code where it is in the folder; for a published framework, judge from what its API demands and what the port had to work around).
6. **Minimum working test** — write, in each version, the smallest passing test that checks that `logger.add(writer, format="{message}")` followed by `logger.debug("a")` writes `"a\n"`. Save each under `_review/`, run it, and report non-blank lines, imports, and names the tester must know.
7. **Fidelity** (ports only; the original is the reference) — does the port still test what each original test tests? List what it checks less, more, or differently.
8. **Where a run starts** — in each version, run the suite from the project folder, then from its `tests/` folder. Then make two copies under `_review/`: one where `pyproject.toml` is removed and an empty `.git` folder is made beside `tests/` (`mkdir .git`), and one with neither. Run each copy from its folder and from its `tests/`. For each run, report what the runner printed about the folder it started from, where each test's id starts, and whether a tester could tell why from the output alone.
9. **Choosing the test files** — in each version, rename the test file to `add_option_format_test.py`, and make the runner collect only files named `*_test.py` through that version's own setting in `pyproject.toml` (find it in the version's rules or the runner's help). Then set that setting wrong three ways, one at a time: the pattern `test*.py`, the string `"*_test.py"` in place of a list, and an empty list. For each, report what the runner printed and did, and whether you could fix the setting from its output alone.
10. **Seeded failures** — in each version, change the expected value in two different tests so that they fail, and run the suite. Report what each run printed, and how quickly the output led you to each failing check.

**Report** (your final answer, plain text, under ~1200 words):
1. A table: the four versions × the ten criteria.
2. Per version, 3–5 lines: strongest and weakest point with evidence.
3. Criteria 8 and 9 again, one line per version each: the score and the one piece of evidence that decided it.
4. Your overall ranking with the weighting stated explicitly.
5. The minimum-working-test counts.
6. Any bug or rough edge you hit (not scored).
```
