# The prompt each toolz rater got (the 1025 form)

The prompt of rounds 2 to 5, of #1010 and of #1015's loguru round, word for word except for the paths, the file it names, and the two tasks that name a test (criteria 3 and 6), which name toolz code instead of loguru code. `<eval>` is the eval folder; rater 2 and 3 got `rater2/` and `rater3/`.

```text
You are one of three independent reviewers. A real pytest test file from the toolz library, `test_itertoolz.py` (12 tests), has been ported into two other Python test forms. Score all three versions. Nobody has told you which version anyone prefers; don't guess, judge only on the evidence in your folder.

**Your folder (the only place you may read):** `<eval>/blind/rater1/`
- `ORIGINAL_pytest/` — the original file.
- `PORT_K/` and `PORT_M/` — the two ports. Each has the port, its own `RUN.md`, `OUTPUT.txt`, `CHECKERS.txt`, and a `NOTES.md` written by the person who ported it (treat the notes as their claims: verify what matters). One port is in a form with its own rules sheet (`FORM.md`).
- `toolz-src/` — the code under test, shared by all three.
Do not read other paths, do not use git or the web. You may write scratch files only under `<your folder>/_review/`.

**Tools:** each folder's `RUN.md` gives the exact run command (a shared venv with pytest 9.2.0 and oxitest 4.0.0 is at `<venv>/`; do not install anything). `ty` is on PATH (`ty check --python <venv>/bin/python …`); pyright via `nix-shell -p pyright --run 'pyright …'` using each folder's `pyrightconfig.json`. Run things; do not rate from reading alone.

**Criteria — score each version 1–5 (5 = best; for both complexity rows, 5 = simplest), each score with concrete evidence (a `file:line`, a quoted line, a measured number, or real output):**
1. **Readability** — how easily a Python developer new to the form scans the file and follows it.
2. **Clarity** — how unambiguous each test's intent is from the code alone: what goes in, what is checked, what is expected, and what a failure on screen means.
3. **Ease of use** — writing and changing tests: do this for real in each version — add one new check to the `tail` test (`tail(2, iter("ABC"))` gives the items `"B"`, `"C"`), then add one new test of your own choosing — and report how long/awkward each was, what the type checkers caught, and how helpful the framework's own errors were.
4. **Complexity for the tester** — concepts, rules and special names a tester must learn.
5. **Complexity of the implementation** — what the form/framework itself must be to support this (for the two ports: read their framework or runner code where it is in the folder; for a published framework, judge from what its API demands and what the port had to work around).
6. **Minimum working test** — write, in each version, the smallest passing test that checks that each window that `sliding_window(2, iter([1, 2, 3, 4]))` yields holds 2 items. Save each under `_review/`, run it, and report non-blank lines, imports, and names the tester must know.
7. **Fidelity** (ports only; the original is the reference) — does the port still test what each original test tests? List what it checks less, more, or differently.

**Report** (your final answer, plain text, under ~1000 words):
1. A table: the three versions × the seven criteria.
2. Per version, 3–5 lines: strongest and weakest point with evidence.
3. Your overall ranking with the weighting stated explicitly.
4. The minimum-working-test counts.
5. Any bug or rough edge you hit (not scored).
```
