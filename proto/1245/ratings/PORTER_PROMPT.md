# The prompt the porter got

Word for word; `<porter>` is the porter's folder and `<venv>` the venv of build A.
The porter read `rules-a/` before two fixes: `TESTS.md` gained `[foreign-framework]`, `[computed-expected]` and the helper's `[second-path]`, and `ROOT.md` the refusal from `tests/`, from its notes.

```text
You port a real pytest test file to a test runner named purba. You know nothing of purba except its written rules; learn it from them and from running it.

**Your folder (the only place you may read):** `<porter>/`
- `rules/` — purba's rules: `TESTS.md` (what a test is and how a run reads), `ROOT.md` (where a run starts), `TEST-FILES.md` (which files are test files). Read all three before you write.
- `ORIGINAL_pytest/` — the original: loguru's `tests/test_add_option_format.py`, with the two fixtures it uses from loguru's conftest (`tests/conftest.py`).
- `port/` — write the port here: `port/tests/test_add_option_format.py`. `port/pyproject.toml` is already there; it holds `[project]` and no `[tool.purba]`. Leave it as it is.
Do not read other paths, do not use git or the web. Do not install anything.

**Tools:** purba is `<venv>/bin/purba`; run it from `port/`. pytest is `<venv>/bin/pytest`, if you want to run the original (from `ORIGINAL_pytest/`, with `-p no:cacheprovider`).

**The port:**
1. Keep what each original test checks: the same inputs and the same expected values. Where purba's rules force a change, make the smallest one, and note it.
2. Every test in the port must pass on purba. Run it from `port/` as `ROOT.md` says a project with no `[tool.purba]` runs, and keep the output.
3. Then check the rules of `ROOT.md` and `TEST-FILES.md` against purba in a copy of `port/` under `<porter>/scratch/`: run once from `tests/`, once without the variable, and once with `[tool.purba]` and a `test-files` value that the rules refuse. Keep each output.
4. Write `port/NOTES.md`, under 400 words: each place the port differs from the original and why (name the rule); each rule you found missing, wrong or unclear while you ported, with the output that showed it; and anything purba printed that surprised you.

**Report** (your final answer, under 300 words): the files you wrote, the count of the run, and the rules you found missing, wrong or unclear.
```
