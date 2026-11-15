# Where test helpers live (#1010, Q4) — throwaway survey

This survey tests one answer of [#1010](https://github.com/kalonji-tools/purba/issues/1010) on real suites before it is posted.
Never merge it.

## Question

purba refuses a test whose act does not take one path. Which code is the test's own, and so watched?

- ⓐ the test module only (purba on `main` today)
- ⓑ the test file's folder and below (prototype 980)
- ⓒ the test module, plus helper modules that a config names

**ⓐ falls if both are true:**

1. Real suites often call a helper of another test module before the first assert, and that helper branches. Then the escape under ⓐ is real and frequent.
2. Folder scope finds few false positives in real layouts, because code under test rarely shares a folder with its tests.

## Verdict: ⓐ held

The first condition is false. Across 9 projects and 4338 tests, 45 tests (1.0 %) call a branching helper of another test module in their act. Another 36 tests (0.8 %) call an asserting helper of another module before their own check. These tests come from 10 helpers in 5 projects. Four projects have no escape at all.

Branching helpers of the test's own module are more common: 161 tests call one. purba on `main` refuses these tests today. So suites do not move branchy helpers out of the module often.

The second condition holds only in part. In 7 of 9 layouts, no code under test sits in a test file's folder. In the 2 flat layouts (six, schedule), the whole package sits there: 126 functions, 54 of them branching. In those layouts, folder scope refuses tests for the `if`s of the library. A run confirmed one such refusal.

## Results

Run on CPython 3.12 with `ast`. Raw output is in `runs/`: one `.json` and `.txt` for each project, and `table.md` for all of them. The `.json` names each test by its file and line, because the spelling gate rewrites some of the projects' own test names.

| project | layout | commit | tests | call a helper of another test module | escape: branching helper | escape: asserting helper before own check | refused today: branching helper of own module | code under test in a test folder (modules / functions / branching) | calls not resolved |
|---|---|---|---|---|---|---|---|---|---|
| loguru | package + `tests/` | `28349be3cd74` | 720 | 151 | 12 | 17 | 9 | 0 | 28.6 % |
| attrs | `src/` + `tests/` | `2a1fb86d4481` | 667 | 44 | 4 | 0 | 8 | 0 | 16.0 % |
| click | `src/` + `tests/` | `e38188e9e205` | 619 | 0 | 0 | 0 | 2 | 0 | 19.5 % |
| requests | `src/` + `tests/` | `3c7baaa05ab4` | 347 | 34 | 28 | 0 | 1 | 0 | 31.0 % |
| dateutil | `src/` + `tests/` | `a78cbd81d5d0` | 1107 | 41 | 1 | 0 | 61 | 0 | 6.0 % |
| pyflakes | tests in the package (`pyflakes/test/`) | `5324946c5d86` | 564 | 507 | 0 | 19 | 1 | 0 | 2.8 % |
| toolz | tests in the package (`toolz/tests/`) | `451af60dec59` | 163 | 0 | 0 | 0 | 10 | 0 | 14.7 % |
| six | flat: `six.py` + `test_six.py` | `c8e394065cd5` | 70 | 0 | 0 | 0 | 1 | 1 / 69 / 20 | 16.2 % |
| schedule | flat: `schedule/` + `test_schedule.py` | `82a43db1b938` | 81 | 0 | 0 | 0 | 68 | 1 / 57 / 34 | 34.7 % |
| **all** | | | **4338** | **777** | **45** | **36** | **161** | **2 / 126 / 54** | 18.8 % (mean) |

The full commits are in `projects.txt`.

The helpers behind each escape:

| project | helper | tests | why it escapes |
|---|---|---|---|
| loguru | `tests/conftest.py` `parse` | 12 | `if strip:` goes one way in each call |
| loguru | `tests/conftest.py` `check_dir` | 17 | asserts, before the test's own check |
| attrs | `tests/strategies.py` `gen_attr_names`, `tests/utils.py` `simple_class` | 4 | an `if`, an `or` |
| requests | `tests/testserver/server.py` `Server`, `TLSServer`, `basic_response_server`, `text_response_server` | 28 | `handler or default` in `Server.__init__`, an `if` in `TLSServer` |
| dateutil | `tests/_common.py` `tz_source_context` | 1 | an `if`; it also sits outside the test's folder, so folder scope misses it too |
| pyflakes | `pyflakes/test/harness.py` `TestCase.flakes` | 19 | asserts with `self.assertEqual`, before the test's own check. 488 more tests have no check of their own, so purba refuses them for that |

Folder scope adds nothing over ⓐ for these helpers except in-folder ones: 23 of the 24 helpers of other test modules sit in the test's folder. Only dateutil's `tests/property/` tests reach a helper above their folder.

## Confirmations (not survey data)

These files are written by hand, in the shape of the real code. No project code ran. The output is in `runs/confirmations.txt`.

- `confirm/escape/`: purba from `main`, `PYTHONPATH=. purba test_env.py`. A copy of loguru's `parse` shape refuses the test when it is in the module, and passes when it is imported from `markup_helpers.py`. This is a real escape.
- `confirm/escape/`: a copy of requests' `override_environ`, a `@contextmanager`, passes in both places. contextlib calls the generator, not the test, so the watch reads it as a value. This also holds under prototype 980's runner.
- `confirm/false-positive/`: prototype 980's `run.py`, in the shape of schedule's `_move_to_next_weekday` and its test. With the library in the test's folder (`flat/`), the case is refused for the library's `if`. With the library in a sibling folder (`split/`), it passes. purba from `main` passes `flat/`.

## What a static survey cannot see

- **18.8 % of calls in acts are not resolved, on average** (2.8 % to 34.7 %). Most are methods of a value: a fixture's object, or an object that the code under test returned (`writer.read()`, `every(10).minutes.do(job)`). So the folder-scope counts for six and schedule are lower bounds. The escape counts miss helpers reached through a fixture's object.
- **A branch in the code is not a branch that goes one way at run time.** A helper with an `if` may take both ways in one call. The counts are an upper bound on refusals.
- **The act is every call before the first `assert` or `self.assert…` in source order.** 1185 tests have no check of their own. For those, the whole body is the act.
- **Fixture bodies are not counted.** In dateutil, 12 tests take a branching fixture of `conftest.py`.
- **A helper driven by contextlib or pytest is free in every scope.** The survey counts it apart: 6 of the 24 helpers of other test modules, and 30 more escaping tests if it counted them.

## Rerun

```sh
uv venv -p 3.12 .venv
bash survey.sh /tmp/helper-scope-clones .venv/bin/python
```

`survey.sh` fetches each project at the commit in `projects.txt` into its own folder, runs `survey.py` on it, and prints the table. `survey.py` reads the files with `ast` only. It never imports or runs them.
