# The blind rating of the line of an implicit root and the errors of `test-files`. Never merge.

Three raters, each in a fresh context, scored two builds of purba beside pytest and oxitest.
The builds differ only in the words of two forms: the line that names a root that `PURBA_IMPLICIT_ROOT` found, and the errors of a wrong `test-files`.
Build A prints short words, and build B prints sentences.
Each rater ran each version, ported from loguru's `test_add_option_format.py`, and did each task of [the prompt](ratings/RATER_PROMPT.md).

**A round compares only with itself.**

## The two forms

5 is best. Each cell is the mean of the three raters, then each rater's score.

| form | the task | pytest | oxitest | purba A | purba B |
|---|---|--:|--:|--:|--:|
| the line of the root | run from the project and from `tests/`, with `pyproject.toml`, with only `.git`, and with neither | 2.67 (3 · 2 · 3) | 2.00 (2 · 2 · 2) | 4.00 (4 · 4 · 4) | **5.00** (5 · 5 · 5) |
| the errors of the patterns of a test file | collect only `*_test.py`, then set `test*.py`, a string, and an empty list | 2.00 (2 · 2 · 2) | 1.33 (2 · 1 · 1) | 3.67 (4 · 3 · 4) | **5.00** (5 · 5 · 5) |

The words that each build prints, from [`rules-a/`](rules-a) and [`rules-b/`](rules-b):

```text
A  ⚠ root ~/only-git, from .git; PURBA_IMPLICIT_ROOT=1; no [tool.purba] found
B  ⚠ root ~/only-git is the folder that holds .git, because PURBA_IMPLICIT_ROOT is set and no pyproject.toml holds [tool.purba]

A  ⚠ root ~/neither, from the working folder; PURBA_IMPLICIT_ROOT=1; no [tool.purba] found; ids start here
B  ⚠ root ~/neither is the working folder, because PURBA_IMPLICIT_ROOT is set and no pyproject.toml holds [tool.purba]; the id of each test starts here, and moves with the working folder

A  pyproject.toml project error: [tool.purba] test-files: test*.py is not a pattern purba takes; nearest: test_*.py
B  pyproject.toml project error: test-files under [tool.purba] holds test*.py, which is none of test_*.py, *_test.py, tests.py and test.py; the nearest is test_*.py

A  pyproject.toml project error: [tool.purba] test-files: a string, not a list
B  pyproject.toml project error: test-files under [tool.purba] is a string; write a list, such as ["*_test.py"]

A  pyproject.toml project error: [tool.purba] test-files: an empty list
B  pyproject.toml project error: test-files under [tool.purba] is an empty list, so no file is a test file; list at least one of test_*.py, *_test.py, tests.py and test.py
```

What decided each score, as the raters wrote it. A ✓ marks a claim that a fresh probe confirmed ([`ratings/redteam.txt`](ratings/redteam.txt), from [`ratings/redteam.sh`](ratings/redteam.sh)).

- **The line of the root.** B gives the reason in each of the six runs, and says that the id moves with the working folder (3 of 3) ✓. A names the step, but its reason is terse (raters 2 and 3), and it does not say that the ids move (rater 2). pytest's and oxitest's `rootdir:` move to `tests/` with no reason when no `pyproject.toml` is there (3 of 3) ✓, and pytest does not read `.git` (raters 1 and 3). Run from `tests/` of the project, pytest's `rootdir:` names the project, and its ids start at `tests/` (3 of 3) ✓. oxitest shows no id on a pass (raters 2 and 3), and an absolute path on a failure (raters 1 and 3).
- **The errors of `test-files`.** B names the item, the closed list, and a fix, so each value was fixed from the output alone (3 of 3) ✓. A refuses each value with exit 4 ✓, but its empty list names no pattern (raters 1 and 2), its line for `test*.py` does not list `*_test.py` (rater 2), and its `nearest: test_*.py` points at the default, not at the `*_test.py` the tester wanted (rater 3). The same nearest is in B's line. pytest takes `["test*.py"]` and runs the files it matches, or none (3 of 3) ✓, takes `[]` with `no tests ran`, exit 5 (raters 1 and 3) ✓, and takes the string (3 of 3) ✓. oxitest takes `["test*.py"]` and `[]` and exits 0 (3 of 3) ✓, and refuses the string, exit 4 ✓. Its key `python_files` is not in `--help`, and each rater found it through the error of an unknown key (3 of 3).

## The rest of the round

The seven criteria of the rating of selection's three forms, word for word, as context, and the seeded failures. For both complexity rows, 5 is simplest.

| criterion | pytest | oxitest | purba A | purba B |
|---|--:|--:|--:|--:|
| readability | 4.00 | 3.00 | 4.00 | 4.00 |
| clarity | 3.67 | 5.00 | 3.67 | 3.67 |
| ease of use | 4.33 | 4.00 | 3.00 | 3.00 |
| complexity for the tester | 3.00 | 2.00 | 3.33 | 3.33 |
| complexity of the implementation | 2.00 | 1.67 | 3.00 | 3.00 |
| smallest test | 5.00 | 4.00 | 4.00 | 4.00 |
| fidelity | reference | 4.67 | 4.33 | 4.33 |
| seeded failures | 4.33 | 4.33 | 4.33 | 4.33 |

Each rater scored A and B the same on every row but the two forms.
Each rater's own ranking, by its own weighting: rater 1 B 48, A 46, pytest 43, oxitest 41 (of 60); rater 2 B 3.92, A 3.67, oxitest 3.42, pytest 3.33; rater 3 pytest 3.90, B 3.75, A 3.55, oxitest 3.40.

## Faults of these builds and this kit, not of the forms

- A test whose expected exception is not raised is blocked by `[second-path] … reaches line 4 of raised, which did not run`, exit 4, and does not fail (raters 1 and 3) ✓. The port checks an exception through a helper, since purba has no form for it yet.
- purba does not capture output, so loguru's default line printed in each smallest test (3 of 3). A failed case prints no file and line (3 of 3).
- The porter read the rules before four fixes, and its [`NOTES.md`](benchmark/purba/NOTES.md) still calls them missing (raters 2 and 3): `TESTS.md` gained `[foreign-framework]`, `[computed-expected]` and the `[second-path]` of a helper, and `ROOT.md` gained the refusal from `tests/`. The notes also name `../scratch/out/`, which is not in the raters' folder (raters 1 and 3); [`ratings/porter/`](ratings/porter) holds it.
- The prompt says the file holds 9 tests; it holds 8 functions and 20 items (raters 1 and 2). The rating of selection's three forms said the same.
- Each venv has a neutral name in a rater's folder, but rater 2 followed the link to `venv-b`. The name does not say which words it prints.
- The root shows an absolute path under `/tmp`, since `~` replaces only the home folder. pytest's and oxitest's `rootdir:` show the same path.

## How it ran

| what | where |
|---|---|
| purba | the code of [`2705ce04`](https://github.com/kalonji-tools/purba/commit/2705ce04a81feba6a289b70773b6195163d7b195) on this branch, from [`main`](https://github.com/kalonji-tools/purba/commit/3ba278b0c8db0d6a22cf87eab38ae2a0254678c1); `mise run build` on a clean tree, once as is (A) and once with `PURBA_PROTO_WORDS=B` (B), a debug build; [`probes/`](probes) shows each word on each build |
| the venvs | CPython 3.12.15, pytest 9.2.0, oxitest 4.0.0, loguru 0.8.0, and one build of purba each |
| purba's rules | [`rules-a/`](rules-a) and [`rules-b/`](rules-b): `TESTS.md` (not rated), `ROOT.md`, `TEST-FILES.md`; the two differ only in the 12 lines they quote |
| the port | [`benchmark/purba/`](benchmark/purba), by a fresh context that read only `rules-a/` and the original ([`ratings/PORTER_PROMPT.md`](ratings/PORTER_PROMPT.md)); 20 cases pass |
| pytest and oxitest | [`benchmark/pytest/`](benchmark/pytest) and [`benchmark/oxitest/`](benchmark/oxitest): loguru's file of the rating of selection's three forms, without dateutil's test, and with `[project]` in each `pyproject.toml` |
| the raters | [`ratings/RATER_PROMPT.md`](ratings/RATER_PROMPT.md), [`ratings/KEY.txt`](ratings/KEY.txt), [`ratings/build_blind.sh`](ratings/build_blind.sh); scores in [`rater1_scores.md`](ratings/rater1_scores.md), [`rater2_scores.md`](ratings/rater2_scores.md) and [`rater3_scores.md.gz`](ratings/rater3_scores.md.gz) (gzipped, since it quotes a misspelling that the typos hook rewrites) |
