# The prompt the porter got

Word for word; `<porter>` is the porter's folder and `<venv>` the shared venv.

```text
You port two real pytest test files to a test runner named purba. You know nothing of purba except its written rules; learn it from them and from running it.

**Your folder (the only place you may read):** `<porter>/`
- `rules/` — purba's rules: `TESTS.md` (what a test is and how a run reads), `QUERY.md`, `TAG.md`, `NAME-NEAR.md`, `RERUN.md`. Read all five before you write.
- `ORIGINAL_pytest/` — the originals: loguru's `tests/test_add_option_format.py` with the two fixtures it uses from loguru's conftest (`tests/conftest.py`), and dateutil's `tests/test_internals.py`, cut to its one test `test_YMD_could_be_day`, which carries the mark `smoke`.
- `port/` — write the port here: `port/tests/test_add_option_format.py` and `port/tests/test_internals.py`. `port/pyproject.toml` is already there.
Do not read other paths, do not use git or the web. Do not install anything.

**Tools:** purba is `<venv>/bin/purba`; run it from `port/`, e.g. `cd <porter>/port && <venv>/bin/purba tests`. pytest is `<venv>/bin/pytest`, if you want to run the originals (from `ORIGINAL_pytest/`, with `-p no:cacheprovider`).

**The port:**
1. Keep what each original test checks: the same inputs and the same expected values. Where purba's rules force a change, make the smallest one, and note it.
2. The dateutil test carries the mark `smoke`; carry it over in purba's form, so that a query by it picks that test.
3. Every test in the port must pass on purba. Run `purba tests` and `purba 'tag(smoke)'` and keep their output.
4. Write `port/NOTES.md`, under 400 words: each place the port differs from the original and why (name the rule); each rule you found missing, wrong or unclear while you ported, with the output that showed it; and anything purba printed that surprised you.

**Report** (your final answer, under 300 words): the files you wrote, the counts of the two runs, and the rules you found missing, wrong or unclear.
```
