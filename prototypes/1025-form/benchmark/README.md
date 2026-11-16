# The blind rating of the 1025 form. Never merge.

Twelve tests of toolz's `toolz/tests/test_itertoolz.py` (`toolz-itertoolz/pytest/`, from the pinned toolz at 451af60), chosen because their checks read lazy products and iterators: `tail`, `peek` and `peekn` hand back an iterator, and `partition_all` and `diff` raise while they are read.
One agent ported them into the form after reading only `FORM.md`, `cases.py`, `examples/` and `run.py` (`toolz-itertoolz/form/`), and another ported them into oxitest 4.0.0 under `strict = "abort"` after reading only oxitest's user documentation (`toolz-itertoolz/oxitest/`).
Three blind raters scored the three versions with the prompt of rounds 2 to 5, #1010 and #1015, word for word but for the paths, the file it names and the two tasks that name code (`ratings/RATER_PROMPT_toolz.md`). The letters were shuffled (`ratings/KEY_toolz.txt`), and each weighted score is recomputed with the original's fidelity counted as 5.
The task of the minimum test asks for a check of each item: each window that `sliding_window(2, iter([1, 2, 3, 4]))` yields holds 2 items.
5 is best; for both complexity rows, 5 is simplest. Compare within this round only: each round's raters score pytest and oxitest differently, and toolz is a new file.

## Toolz `test_itertoolz.py`

| | readability | clarity | ease of use | tester | implementation | smallest test | fidelity | weighted, rater 1 · 2 · 3 | mean |
|---|--:|--:|--:|--:|--:|--:|--:|---|--:|
| pytest | 4.33 | 3.0 | 5.0 | 5.0 | 3.67 | 5.0 | reference | 4.55 · 4.35 · 4.35 | 4.42 |
| oxitest | 4.0 | 4.0 | 4.0 | 4.0 | 3.0 | 4.0 | 5.0 | 4.15 · 4.15 · 4.05 | 4.12 |
| **the form** | 3.0 | 4.33 | 2.67 | 2.0 | 2.0 | 3.33 | 4.0 | 3.00 · 3.25 · 3.30 | **3.18** |

The gap to pytest is −1.23 and to oxitest −0.93. Each rater ranked the form last. The form scored highest of the three on clarity only.

- **What the raters counted for the form (3 of 3):** the failure screen. A broken `all()` lists each item with its reading, `[0]  len(w)  2     w  (1, 2)`, and a count, `3 of 3 items break  len(w) == 3`. The refusals name the fix: "the expected value computes `list('BC')` … write a literal". A misspelled input of a case is caught by ty and by the runner. Each raise also checks its message, and `==` checks the type.
- **The rule of this round that cost the most (3 of 3):** each rater's first minimum test in the form was the pytest one-liner, `assert all(len(w) == 2 for w in sliding_window(2, iter([1, 2, 3, 4])))`, and the runner refused it: "reads `sliding_window(…)`, a zip that one reading empties; make it a list in the act". The second try put the windows in a list in the act and passed. Each rater also wrote that `FORM.md:39` ("An iterator that the check makes itself is not seen") reads as if that one-liner passes, and that "a zip" names the type inside toolz, not what the tester wrote.
- **Against the form, from earlier rounds' rules (3 of 3):** 12 tests became 16 tests and 24 cases, with each expected raise in a test of its own and its cases at module level; `FORM.md` at 200 lines and `run.py` at 3,265; `list("BC")` refused as a computed expected value; a test written the pytest way, with checks before an expected raise, refused as an assert in the act (rater 2, three times).
- **oxitest:** the closest port: each assert gains a message, and a missing message stops the whole run with exit 3. The documented "module-level definition" strict check never fired (the porter and raters 1, 2 and 3 confirmed it).
- **The original:** `test_itertoolz.py:152` compares two values and keeps nothing, so it checks nothing; pyright flags it, and both ports turned it into a check.

## Faults of this runner, not of the form

- A failing run counted "0 blocked" while four checks showed `⊘ blocked`: the count names cases (rater 2).
- Each failed or refused case prints the line of the seed of `random`, even when the test reads no `random` (raters 1 and 2).
- An assert in the act is refused under the label `path`, with a text about a function handed to the code (the porter and rater 2).
- A `try` that got another raise names no expected type (rater 1).

## Files

- `toolz-itertoolz/pytest/` (the original), `toolz-itertoolz/form/`, `toolz-itertoolz/oxitest/`: each version with its `RUN.md`, `OUTPUT.txt`, `CHECKERS.txt` and the porter's `NOTES.md`; `toolz-itertoolz/toolz-src/` and `LICENSE.toolz`: the code under test, unchanged.
- `ratings/`: the build script of the blind folders, the key, the prompt, and each rater's scores, condensed.

The runs used CPython 3.12.15, pytest 9.2.0 and oxitest 4.0.0.
