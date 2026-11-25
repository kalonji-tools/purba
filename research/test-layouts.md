# Layouts of test files in Python projects, and which of them purba's selection misses

Research for [Which layouts of test files do Python projects use, such as Django's, and which of them does purba's selection miss?](https://github.com/kalonji-tools/purba/issues/1239)
Measured on 2026-11-25, against purba `main` at `cfc09617`, on one Linux machine.
This file decides nothing. It surveys the layouts, measures them in 18 pinned repositories, and weighs the changes against purba's records.
It builds on [the findings of #1225](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md) and [the findings of #1227](https://github.com/kalonji-tools/purba/blob/ceede296e32cc271ee090f2dcbcf08243d861be4/research/selection-strategies.md), and does not repeat their counts.

## Summary

purba's file patterns already match most test files in the layouts surveyed. Django is the exception at the file layer, and the file layer is the smallest of its gaps. A Django test is a method of a `TestCase` class, and it needs Django's settings and a test database before it runs. purba's selection reads neither a method nor a setup. So no change of a file pattern lets purba run a Django suite. The other layouts differ mostly at the root and the run, not at the file name.

Six findings matter most for purba.

- **Django's own files miss purba's patterns.** `startapp` writes `polls/tests.py` [run]. Django's runner finds `test*.py` from the working folder, and only in folders with `__init__.py` [run]. purba's patterns miss `tests.py` and `tests_*.py` [run]. In Django's repository, 216 files hold a test and match `test*.py`, and not purba's patterns. 209 of them are named `tests.py` [run].
- **A Django test is a method, never a function.** Django's runner skipped a top-level `def test_top_level()` in `polls/tests.py` [run]. In Django and the three Django applications, 978 of the 995 files that purba's patterns match hold only methods. They hold 27,204 `test*` methods and 14 `test*` functions [run].
- **A Django test needs a setup that purba does not give.** Without settings, a `SimpleTestCase` stops with `ImproperlyConfigured` [run]. With settings and no `django.setup()`, the import stops with `AppRegistryNotReady` [run]. With both and no test database, a model test fails with `no such table` [run]. [purba never loads a plugin](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md?plain=1#L39), and names a Django database as a fixture that each test names [L46](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md?plain=1#L46).
- **Outside Django, unittest's `test*.py` adds few test files.** In the 9 suites, it adds 1 file, with no test. In Flask, FastAPI, CPython, NumPy and LangChain together, it adds 28 files, and 18 of them hold a test. These 18 are CPython's `testmock/testmock.py` kind and NumPy's `tools/swig/test/testArray.py` kind [run]. It also adds library modules such as `src/flask/testing.py` and `rest_framework/test.py` [run].
- **No repository names a root by `[tool.purba]`.** 0 of the 52 `pyproject.toml` files in the 18 repositories hold it [run]. So each layout reaches a root only through the steps that `PURBA_IMPLICIT_ROOT` opens. From the root of the repository, 14 repositories reach step ②, and 4 reach step ③ [run, a matcher of the ladder]. `startproject` writes no `pyproject.toml` [run], so a new Django project reaches step ③ or ④.
- **Several projects in one repository give a root for each project.** LangChain holds 22 `pyproject.toml`, and none at its root [run]. From the root, the ladder reaches step ③ and takes all 624 test files. From the folder of one test file, it reaches step ② at that file's own project, 22 roots in all [run, a matcher of the ladder].

judgement: purba can work with Django only if purba runs a method of a class, and gives Django's setup through a fixture. A file pattern would come last, after those two. Section 5 weighs each candidate.

## How to read this file

- **[run]** means I ran it in this session, on the version named. **[source]** means the tool's code or documentation says so, at the pinned version or commit, and I did not run it. **judgement** marks a claim that no source states.
- **purba's stage one is not built.** No line here says what purba prints. A row that applies purba's patterns or its ladder uses my matcher in `measure.py` or in a probe. It is a matcher, not purba.
- **P** is purba's pattern of a [test file](https://github.com/kalonji-tools/purba/blob/cfc09617/CONTEXT.md?plain=1#L359-L361): a `.py` file whose name matches `test_*.py` or `*_test.py`, by `fnmatch.fnmatchcase`.
- **U** is unittest's default pattern, `test*.py`, applied to the file's name only. **U and a package path** adds unittest's rule that each folder down to the file holds `__init__.py`.
- **The four layers.** *File*: does P match the file? *Test*: is each test a function at the top level, [as purba's term says](https://github.com/kalonji-tools/purba/blob/cfc09617/CONTEXT.md?plain=1#L353)? *Run*: does the test need a setup that purba does not give yet? *Root*: which step of the ladder finds the root?
- **The ladder** is the one that the brief of this research gives for stage one. No record holds it yet. ① a `pyproject.toml` that holds `[tool.purba]`. If none, and `PURBA_IMPLICIT_ROOT` is set: ② a `pyproject.toml` that holds `[project]`; ③ the folder that holds `.git`; ④ the working folder. The walk goes up from the working folder, and [stops at the folder that holds `.git`](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-reads-configuration-from-six-levels.md?plain=1#L67). My matcher takes the nearest folder at each step.
- A count answers only its own matcher. Each table row names its matcher.

## Method

### Versions

| tool | version | how I ran it | pinned source |
|---|---|---|---|
| Django | 6.1.3 | a `uv` venv, CPython 3.13.13, `-I` | [`test/runner.py`](https://github.com/django/django/blob/6.1.3/django/test/runner.py#L784-L786), [`overview.txt`](https://github.com/django/django/blob/6.1.3/docs/topics/testing/overview.txt#L44-L48) |
| pytest | 9.2.0 | the same venv | [`python.py`](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/python.py#L98-L101), [`doctest.py`](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/doctest.py#L98-L102) |
| unittest, doctest | CPython 3.13.13 | the same venv | [`loader.py`](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/loader.py#L229) |
| CPython's `regrtest` | 3.13.13 | not run: the `uv` build ships no `Lib/test` | [`findtests.py`](https://github.com/python/cpython/blob/v3.13.13/Lib/test/libregrtest/findtests.py#L38-L61) |
| `measure.py` | CPython 3.14.7, free-threaded | `mise exec python@3.14`, `-I` | — |

The pytest tag `9.2.0` resolves to `ae1d0ff57b8bd4252da9ade7353523ac0ac73281`.

### Repositories

`repos.tsv` pins 18 repositories. The 9 suites are those of [the cost of the rules](https://github.com/kalonji-tools/purba/blob/8e712dbf/prototypes/1015-rules-cost/projects.txt). The others are Django's repository, three Django applications, and five repositories for the other layouts. `clone.sh` fetches each at its ref, one commit deep. It stops when the commit differs from the row [run].

| name | ref | commit | layout it stands for |
|---|---|---|---|
| django | `6.1.3` | `f16e25dcb9fe55734ba4ea2b00ac34405b7df74a` | Django's own suite: `tests/<app>/tests.py` |
| drf | `3.18.3` | `1af5b55758ac514c492ed3a5fc1a6fb244cdb257` | a Django application, run by pytest |
| wagtail | `v8.1` | `99924bbfbfd5075f9d603771981111ffd9e4a1c1` | a Django application, tests inside the package |
| debug-toolbar | `8.0.0` | `85b98c6a93e1ff906f07545e98d8e4085424d005` | a Django application, `tests/` beside it |
| flask | `3.1.3` | `22d924701a6ae2e4cd01e9a15bbaf3946094af65` | a `src` layout |
| fastapi | `0.165.0` | `05e6df7e906c3decba377c37548019973f40bf15` | a flat layout |
| cpython | `v3.13.13` | `01104ce1beb3135c2e0c01ec835b994c1f55a1c0` | `Lib/test/` |
| numpy | `v2.5.5` | `afcbc6d21a88a7b59c5bfe115198e24ed51b3dae` | tests inside the package |
| langchain | `87ad2d71` | `87ad2d71e1351fc1508483610bb6c965b3bbdffe` | several projects in one repository |

The 9 suites are at the commits of `projects.txt`.

### Probes and counts

Each probe is `research/test-layouts/<name>/run.sh`. It writes a small tree into an empty folder, and runs the tool there. `out.txt` beside it holds this session's output. The folder's path is replaced by `<dir>`, and the spaces at the end of a line are removed.

```sh
R=research/test-layouts
bash $R/clone.sh $R/repos.tsv <empty dir>
bash $R/django/run.sh <python 3.13.13 with django 6.1.3> <empty dir>
bash $R/layouts/run.sh <python 3.13.13 with pytest 9.2.0> <empty dir>
mise exec python@3.14 -- python -I $R/measure.py <clones dir> $R/repos.tsv $R
```

`measure.py` reads each file that git tracks, and skips each hidden folder, as stage one does. It parses each `.py` file with `ast`, and reads each `pyproject.toml` with `tomllib`. It imports nothing from the repositories. It writes `counts.txt`, `summary.tsv` and `files.tsv.gz`, one row for each file that P or U matches.

## 1. Django

### 1.1 What `startapp` writes, and where

`django-admin startproject mysite .` and `manage.py startapp polls` write 13 files [run, `django/out.txt` step 1]. The one test file is `polls/tests.py`. It holds `from django.test import TestCase` and a comment, and no test [run]. The template is [`app_template/tests.py-tpl` L1-L3](https://github.com/django/django/blob/6.1.3/django/conf/app_template/tests.py-tpl#L1-L3). Neither command writes a `pyproject.toml` [run].

### 1.2 What Django's runner finds, and from which folder

| probe | what ran | source |
|---|---|---|
| `manage.py test`, from the folder of `manage.py` | `polls/tests.py`, `polls/test_views.py`, `polls/tests_extra.py`, `shop/tests/test_models.py`: 4 tests [run, step 3] | [`runner.py` L784-L786](https://github.com/django/django/blob/6.1.3/django/test/runner.py#L784-L786): `-p`, default `test*.py` |
| the same run: files that did not run | `polls/models_test.py`, `polls/check_models.py`, and `polls/nopkg/test_y.py`, whose folder has no `__init__.py` [run, step 3] | [`loader.py` L421-L423](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/loader.py#L421-L423) |
| `manage.py test -p '*_test.py'` | only `polls/models_test.py` [run, step 4] | [`overview.txt` L116-L123](https://github.com/django/django/blob/6.1.3/docs/topics/testing/overview.txt#L116-L123) shows `--pattern="tests_*.py"` |
| `manage.py test`, from the folder `polls` | 3 tests: the run leaves out `shop` [run, step 6] | [`runner.py` L963-L964](https://github.com/django/django/blob/6.1.3/django/test/runner.py#L963-L964): with no label, the label is `.`; [`overview.txt` L86-L89](https://github.com/django/django/blob/6.1.3/docs/topics/testing/overview.txt#L86-L89) |
| Django's own suite | `tests/runtests.py` takes each subfolder of `tests/` that holds `__init__.py` as an app and a label | [`runtests.py` L108-L137](https://github.com/django/django/blob/6.1.3/tests/runtests.py#L108-L137) |

Django's runner reads the label `.` as the working folder. purba's ladder reads the working folder only to start its walk. judgement: a Django tester who runs from `polls/` expects only `polls`.

### 1.3 The layouts that Django's documentation shows

| layout | what the documentation says | source |
|---|---|---|
| one `tests.py` in each app | "The default startapp template creates a `tests.py` file in the new application" | [`overview.txt` L52-L60](https://github.com/django/django/blob/6.1.3/docs/topics/testing/overview.txt#L52-L60) |
| a `tests/` package in each app | the same note: split it into `test_models.py`, `test_views.py`, `test_forms.py` | [`overview.txt` L52-L60](https://github.com/django/django/blob/6.1.3/docs/topics/testing/overview.txt#L52-L60) |
| `tests_*.py` | the example of `--pattern` | [`overview.txt` L116-L123](https://github.com/django/django/blob/6.1.3/docs/topics/testing/overview.txt#L116-L123) |
| a reusable app: `tests/` beside the app, with `runtests.py`, `tests/test_settings.py` and `tests/tests.py` | `runtests.py` sets `DJANGO_SETTINGS_MODULE`, calls `django.setup()`, and runs the label `tests` | [`advanced.txt` L402-L446](https://github.com/django/django/blob/6.1.3/docs/topics/testing/advanced.txt#L402-L446) |

The reusable layout names its settings `tests/test_settings.py`. P matches that name, and the file holds no test. Django's own suite does the same with [`tests/test_sqlite.py`](https://github.com/django/django/blob/6.1.3/docs/internals/contributing/writing-code/unit-tests.txt#L162-L164). judgement: under purba, each such file is a test file that holds no test.

### 1.4 The shape of a test

A Django test is a method of a subclass of `unittest.TestCase` [source, [`overview.txt` L44-L48](https://github.com/django/django/blob/6.1.3/docs/topics/testing/overview.txt#L44-L48)]. The runner skipped `def test_top_level()`, a function at the top level of `polls/tests.py`. From that file, it ran only `PollTests.test_method` [run, step 3]. The function raises when called, and each of the 4 tests of the run printed `ok`.

### 1.5 What a test needs before it runs

| probe | result | source |
|---|---|---|
| `unittest` on `polls.tests`, with no settings | `ImproperlyConfigured: Requested setting DATABASES, but settings are not configured` [run, step 7] | — |
| `unittest` on `shop.tests.test_models`, with `DJANGO_SETTINGS_MODULE` and no `django.setup()` | `AppRegistryNotReady: Apps aren't loaded yet.` [run, step 8] | — |
| the same, after `django.setup()`, with no test database | `OperationalError: no such table: shop_book` [run, step 9] | — |
| `manage.py test` | `Creating test database for alias 'default'`, then the run [run, step 3] | [`overview.txt` L151-L158](https://github.com/django/django/blob/6.1.3/docs/topics/testing/overview.txt#L151-L158); [`advanced.txt` L489-L500](https://github.com/django/django/blob/6.1.3/docs/topics/testing/advanced.txt#L489-L500) lists setup, then discovery, then the test databases, then `migrate` |

The three Django applications each set the settings in their own way [source]:

- Wagtail sets [`wagtail.test.settings`](https://github.com/wagtail/wagtail/blob/v8.1/runtests.py#L11), and runs Django's runner from the root ([L99-L104](https://github.com/wagtail/wagtail/blob/v8.1/runtests.py#L99-L104)).
- django-debug-toolbar sets `tests.settings`, and runs `python -m django test tests` ([`Makefile` L19-L21](https://github.com/django-commons/django-debug-toolbar/blob/8.0.0/Makefile#L19-L21)).
- DRF runs pytest on `tests` ([`runtests.py` L52](https://github.com/encode/django-rest-framework/blob/3.18.3/runtests.py#L52)). Its `conftest.py` calls `settings.configure()` in `pytest_configure` ([L40-L60](https://github.com/encode/django-rest-framework/blob/3.18.3/tests/conftest.py#L40-L60)).

## 2. The other layouts

| layout | the example | what the tool's documentation says | source |
|---|---|---|---|
| `tests/` beside the package, flat | loguru, DRF, debug-toolbar, FastAPI, Django's repository | FastAPI: `test_main.py` can also sit beside `main.py` in the package | [`testing.md` L83-L91](https://github.com/fastapi/fastapi/blob/0.165.0/docs/en/docs/tutorial/testing.md?plain=1#L83-L91) |
| `tests/` beside a `src` layout | attrs, click, requests, dateutil, Flask | pytest: "Tests outside application code"; Flask: `tests` is next to the package, not inside it | [`goodpractices.rst` L129-L148](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/explanation/goodpractices.rst?plain=1#L129-L148), [`tests.rst` L39-L45](https://github.com/pallets/flask/blob/3.1.3/docs/tutorial/tests.rst?plain=1#L39-L45) |
| tests inside the package | toolz, pyflakes, NumPy, Wagtail | pytest: "Tests as part of application code", `mypkg/tests/test_app.py` | [`goodpractices.rst` L217-L235](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/explanation/goodpractices.rst?plain=1#L217-L235) |
| one test file at the root | six (`test_six.py`), schedule (`test_schedule.py`) | — | the trees [run] |
| one `tests.py` or `test.py` at the root | the probe | pytest collects neither; unittest loads only the `TestCase` of `tests.py` | [run, `layouts/out.txt` steps 1 and 4] |
| a doctest in a module | requests: `addopts = "--doctest-modules"` | pytest collects a module's doctests only with `--doctest-modules` | [`pyproject.toml` L112](https://github.com/psf/requests/blob/3c7baaa05ab4aa49cc819ce3a819d260de19c157/pyproject.toml#L112), [`doctest.py` L83](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/doctest.py#L83) |
| a doctest in a text file | the probe's `docs/usage.txt` | pytest's `--doctest-glob`, default `test*.txt` | [`doctest.py` L98-L102](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/doctest.py#L98-L102), [L152](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/doctest.py#L152) |
| several projects in one repository | LangChain: `libs/core`, `libs/langchain`, 16 under `libs/partners`, and 4 others | each folder holds its own `pyproject.toml` with `[project]` | [`libs/core/pyproject.toml` L5-L6](https://github.com/langchain-ai/langchain/blob/87ad2d71e1351fc1508483610bb6c965b3bbdffe/libs/core/pyproject.toml#L5-L6) |
| CPython's `Lib/test/` | `test_*.py` modules, and `test_*` packages | `regrtest` takes each name in `Lib/test/` that starts with `test_`, a module or a folder; it splits 10 folders into their own `test_*.py` | [`findtests.py` L18-L29](https://github.com/python/cpython/blob/v3.13.13/Lib/test/libregrtest/findtests.py#L18-L29), [L38-L61](https://github.com/python/cpython/blob/v3.13.13/Lib/test/libregrtest/findtests.py#L38-L61) |

Three layouts appeared in the trees that no row of the issue holds.

- **A test package whose `__init__.py` holds the tests.** `Lib/test/test_dataclasses/__init__.py` holds the `test*` methods of that package [run]. In CPython, 9 `__init__.py` files hold a test [run, `counts.txt`]. P never matches `__init__.py`.
- **`test<Name>.py`, with no `_`.** CPython's `Lib/test/test_unittest/testmock/testmock.py` and 9 more, and NumPy's `tools/swig/test/testArray.py` and 7 more [run, `files.tsv.gz`]. U matches them, and P does not.
- **A library module whose name starts with `test`.** `src/flask/testing.py`, `rest_framework/test.py`, `fastapi/testclient.py`, `src/click/testing.py`, `django/test/testcases.py` [run, `files.tsv.gz`]. U matches each. unittest imports each as a test module, and finds no test in it.

The doctest probe shows each runner [run, `layouts/out.txt`]. pytest with no option collects 2 tests, from `tests/test_lend.py` and `tests/lend_test.py`. `--doctest-modules` adds `pkg/calc.py::pkg.calc.add`. `--doctest-glob='*.txt'` adds `docs/usage.txt`. `python -m doctest` passes both files.

A `tests.py` beside a `tests/` package stops unittest's discovery: `ImportError: 'tests' module incorrectly imported` [run, `layouts/out.txt` step 5].

## 3. The four layers for each layout

✓ means the layer meets purba as the records stand. ✗ means it does not. ~ means a part of the files meets it, and the count names the part. A ✓ or ✗ in the file and test columns comes from the probe or from `files.tsv.gz`. Each root step is my matcher of the ladder, from the root of the repository, with `PURBA_IMPLICIT_ROOT` set; without it, every row has no root.

| layout | example | file: P matches | test: a function at the top level | run: no setup that purba lacks | root step |
|---|---|---|---|---|---|
| Django app, `startapp` default: `polls/tests.py` | the probe | ✗ [run] | ✗ a `TestCase` method [run] | ✗ settings, `django.setup()`, test database [run] | ③, or ④ with no `.git`: no `pyproject.toml` [run] |
| Django app with a `tests/` package: `shop/tests/test_models.py` | the probe | ✓ [run] | ✗ [run] | ✗ [run] | ③ or ④ [run] |
| Django `tests_*.py` | the probe; Django's repository, 7 files | ✗ [run] | ✗ | ✗ | as above |
| Django reusable app: `tests/` beside the app, `runtests.py` | debug-toolbar; Django's documentation | ✓ for `test_*.py`, ✗ for `tests/tests.py`; `test_settings.py` matches and holds no test | ✗ 31 of 31 files hold only methods [run] | ✗ [source] | ② [run] |
| Django's own suite: `tests/<app>/tests.py` | django | ~ P 628 of U 852 [run] | ✗ 619 of 628 hold only methods [run] | ✗ `test_sqlite` settings, `runtests.py` [source] | ② [run] |
| Django app, tests inside the package | wagtail | ~ P 276 of U 291 [run] | ✗ 273 of 276 hold only methods [run] | ✗ [source] | ② [run] |
| Django app run by pytest | DRF | ✓ 60 of 61 U; the other is `rest_framework/test.py` [run] | ✗ 55 of 60 hold only methods [run] | ✗ `settings.configure()` in a pytest hook; 32 files import pytest [run, source] | ② [run] |
| `tests/` beside the package, flat | loguru, FastAPI | ✓ [run] | ✓ loguru 50 of 50, FastAPI 562 of 567 hold only functions [run] | ✗ pytest fixtures: loguru 39 of 50, FastAPI 336 of 568 import pytest [run] | ② [run] |
| `tests/` beside a `src` layout | attrs, click, requests, dateutil, Flask | ✓ [run] | ~ attrs 17 of 26 files hold only methods; click 43 of 43 hold only functions [run] | ✗ pytest fixtures in `conftest.py`: Flask's `app` and `client` [source] | ② [run] |
| tests inside the package | toolz, pyflakes, NumPy | ✓; NumPy also has 8 `test<Name>.py` with tests [run] | ~ toolz 28 of 29 functions only; pyflakes 13 of 13 methods only; NumPy 83 of 181 methods only [run] | ~ NumPy: 148 of 190 import pytest [run] | ② for toolz and NumPy, ③ for pyflakes [run] |
| one test file at the root | six, schedule | ✓ [run] | ~ six holds both; schedule holds only methods [run] | ✓ judgement | ③ for six, ② for schedule [run] |
| one `tests.py` or `test.py` at the root | the probe | ✗ [run] | ✓ for a function, ✗ for a `TestCase` [run] | ✓ judgement | ③ or ④ |
| a doctest in a module | requests, the probe | ✗ unless the module's name matches P [run] | ✗ a docstring, not a function [run] | ✗ a runner of doctests [run] | — |
| a doctest in a text file | the probe | ✗ not a `.py` file [run] | ✗ [run] | ✗ [run] | — |
| several projects in one repository | LangChain | ✓ 624 of 624 [run] | ✓ 536 of 624 hold only functions [run] | ✗ 374 of 624 import pytest; 19 `conftest.py` [run] | ③ from the root of the repository; ② from inside a project, 22 roots [run] |
| CPython's `Lib/test/` | cpython | ~ P 774 of U 789; 9 `__init__.py` with tests [run] | ✗ 721 of 774 hold only methods [run] | ~ `test.support` and `regrtest`'s resources [source] | ③ [run] |

judgement: three columns block Django. The file column blocks least: 628 of the 852 U files in Django's repository already match P.

## 4. Counts

Each count is [run], from `counts.txt` and `summary.tsv`. The 9 suites give P 189 and 1,899 test functions, as #1225 counted.

### 4.1 Files that U finds and P does not

Matcher: U, and not P. "With a test" means the file holds a `test*` function at the top level or a `test*` method of a top-level class.

| repository | U | P | U, not P | `tests.py` | `tests_*.py` | `test.py` | other `test*.py` | with a test | P, not U |
|---|---|---|---|---|---|---|---|---|---|
| the 9 suites | 190 | 189 | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| django | 852 | 628 | 224 | 211 | 7 | 2 | 4 | 216 | 0 |
| drf | 61 | 60 | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| wagtail | 291 | 276 | 15 | 14 | 0 | 0 | 1 | 14 | 0 |
| debug-toolbar | 31 | 31 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| flask | 28 | 27 | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| fastapi | 568 | 567 | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| cpython | 789 | 774 | 17 | 0 | 0 | 1 | 16 | 10 | 2 |
| numpy | 190 | 181 | 9 | 0 | 0 | 0 | 9 | 8 | 0 |
| langchain | 624 | 624 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

The 9 suites' one file is `src/click/testing.py`. In Django's repository, 209 of the 211 `tests.py` hold a test. One `tests_*.py` does not parse on purpose: `tests/test_runner_apps/tagged/tests_syntax_error.py`. CPython's 2 files of P, not U, are `torture_test.py` and `a_test.py`.

Matcher: U and a package path. In Django's repository, 6 of the 852 U files sit under folders that each hold `__init__.py` [run]. `tests/` has no `__init__.py`, so `unittest discover` from the root misses the suite. Django runs it through `runtests.py` (section 1.2).

### 4.2 The shape of the tests in the files that P matches

Matcher: in each P file, a `def` or `async def` named `test*` in the module's body; and a `def` named `test*` in the body of a class in the module's body, of any base.

| group | P files | functions only | both | methods only | no test | `test*` functions | `test*` methods |
|---|---|---|---|---|---|---|---|
| the 9 suites | 189 | 142 | 14 | 33 | 0 | 1,899 | 2,505 |
| Django and its 3 applications | 995 | 2 | 8 | 978 | 7 | 14 | 27,204 |
| Flask, FastAPI, CPython, NumPy, LangChain | 2,173 | 1,159 | 97 | 844 | 73 | 9,655 | 34,698 |

### 4.3 Imports and `conftest.py`

Matcher: in each P or U file, an `import` or a `from … import` anywhere whose first name is `django`, or is `pytest` or `_pytest`. [A test module imports no other test framework](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/a-test-module-imports-no-other-test-framework.md?plain=1#L33) refuses only the second.

| group | P or U files | import django | import pytest | `conftest.py` files |
|---|---|---|---|---|
| the 9 suites | 190 | 0 | 119 | 5 |
| Django and its 3 applications | 1,235 | 1,218 | 32, all in DRF | 1, in DRF |
| Flask, FastAPI, CPython, NumPy, LangChain | 2,199 | 0 | 878 | 28 |

### 4.4 Roots

Matcher: my ladder of the brief, with `PURBA_IMPLICIT_ROOT` set, on each repository's own tree; `.git` is at each clone's root.

| count | value |
|---|---|
| `pyproject.toml` files in the 18 repositories | 52 |
| of them, with `[tool.purba]` | 0 |
| repositories whose root is step ② from their root | 14 |
| repositories whose root is step ③ from their root | 4: pyflakes, six, cpython, langchain |
| distinct roots, from the folder of each P or U file | langchain 22, flask 3, cpython 2, each other 1 |

Flask's 3 roots are the root and `examples/tutorial` and `examples/javascript`. click holds 10 `pyproject.toml` under `examples/`, and no test file sits under them.

### 4.5 Doctests

Matcher: a root `pyproject.toml`, `setup.cfg`, `tox.ini` or `pytest.ini` that names `--doctest-modules`, `--doctest-glob` or a `doctest_` key; and, by `git grep`, a `.py` file that calls `DocTestSuite`, `DocFileSuite`, `doctest.testmod` or `doctest.testfile`.

| repository | names a doctest option | calls the doctest API |
|---|---|---|
| requests | `--doctest-modules` | — |
| numpy | `doctest_optionflags` in `pytest.ini` | — |
| cpython | — | 47 files |
| django, toolz | — | 1 file each |

CPython runs its doctests from `load_tests` hooks: 75 files in `Lib/test/` define one [run, `git grep`]. judgement: under purba, a doctest reaches a run only when a test function calls the doctest API.

### 4.6 Names near a pattern

Matcher: a `.py` file that P misses, whose first word, up to the first `_`, or last word, after the last `_`, is at most one edit from `test`, case ignored. It moves [the rule of a name near `test`](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md?plain=1#L79-L80) from a function to a file, and adds the last word for `*_test.py`. It is not a check that any record holds.

| group | near files | with a test | with no test |
|---|---|---|---|
| the 9 suites | 0 | 0 | 0 |
| Django and its 3 applications | 252 | 231 | 21 |
| Flask, FastAPI, CPython, NumPy, LangChain | 20 | 8 | 12 |

The 33 with no test include `django/utils/text.py`, `rest_framework/templatetags/rest_framework.py`, `…/commands/subparser_dest.py`, `wagtail/test/utils/page_tests.py` and `libs/langchain/langchain_classic/text_splitter.py` [run, `counts.txt`].

## 5. The changes, weighed

Each candidate is judgement, weighed against the records at `cfc09617`. Each one that adds a pattern or a line changes a form that a tester meets. So each one owes [a blind rating](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/a-form-that-a-tester-meets-is-rated-blind-before-it-is-decided.md?plain=1#L28) before the person decides.

### 5.1 At the file layer

**Add `tests.py` and `tests_*.py` to the patterns.**

```text
test_*.py   *_test.py   tests.py   tests_*.py
```

- It gains 215 test-holding files in Django's repository, and 14 in Wagtail [run, 4.1]. It gains 0 in the 9 suites and in the five other repositories.
- It runs no test in them today. Each of the 229 holds only methods [run, `files.tsv.gz`], and selection reads only functions at the top level.
- It does not reach `test<Name>.py`: CPython's 10 and NumPy's 8.
- It changes the term [test file](https://github.com/kalonji-tools/purba/blob/cfc09617/CONTEXT.md?plain=1#L359-L361), and the record's line [L77](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md?plain=1#L77).

**Take unittest's `test*.py`.**

```text
test*.py   *_test.py
```

- It gains each file of the row above, and the 18 `test<Name>.py` with a test in CPython and NumPy [run, 4.1].
- It also gains 21 files with no test, one of which does not parse on purpose: `src/flask/testing.py`, `rest_framework/test.py`, `fastapi/testclient.py`, `django/test/testcases.py` and kin [run, `files.tsv.gz`]. #1225 found the same cost for a folder: a marker that catches a helper turns it into a unit.
- Each library module it gains is code under test. judgement: stage two parses each, and finds no test. Each of its top-level functions then meets the name-near check of [L79](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md?plain=1#L79). I did not count the functions that the check would name.

**Report a file whose name is near a pattern.**

```text
polls/tests.py   (near test_*.py: a notice, or a project error)
```

- With an error, Django's repository would rename 209 `tests.py` and the other near files. The 9 suites would rename none [run, 4.6].
- It flags 33 files with no test, such as `django/utils/text.py` and `subparser_dest.py` [run, 4.6]. Those are false positives of my matcher. A helper takes another name under the function rule ([L132](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md?plain=1#L132)). A module of the code under test cannot take another name without a change to its importers.
- A notice that leaves the exit code alone was considered and lost for a function's name: [L38](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md?plain=1#L38) says a run in CI passes with a test that never ran.
- It answers the downside of [L133](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md?plain=1#L133), that a misspelled test file is silent.

**Change nothing.**

```text
test_*.py   *_test.py
```

- A Django project with only `tests.py` gives a run in which no case ran. [Such a run exits 2](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md?plain=1#L56-L69), so CI does not read it as clean.
- A project with some `test_*.py` and some `tests.py` runs only the first kind, and says nothing about the second. In Django's repository, that is 216 of the 852 U files [run, 4.1].
- `tests/test_settings.py` and `tests/test_sqlite.py` stay test files with no test. judgement: stage two finds no test in them, and the run is unchanged.

### 5.2 At the test layer

purba's [test](https://github.com/kalonji-tools/purba/blob/cfc09617/CONTEXT.md?plain=1#L353) is a function at the top level. 978 of the 995 P files in the Django group hold only methods [run, 4.2]. judgement: this layer blocks Django before the file layer does. A change here is not a change of selection's patterns. It reopens the term "test", and the decorators, the inputs and the fixtures that hang on it. No candidate of the issue reaches it.

### 5.3 At the run layer

[purba never loads a plugin](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md?plain=1#L39). [Its downside](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md?plain=1#L46) already names a Django database as a fixture that each test names. The probe shows the three steps that such a fixture owes: the settings, `django.setup()`, and a test database [run, 1.5]. judgement: the design of fixtures decides this, not selection.

### 5.4 At the root

No repository holds `[tool.purba]` [run, 4.4]. So each layout reaches a root only with `PURBA_IMPLICIT_ROOT` set.

- **A Django project from `startproject`.** It has no `pyproject.toml` [run], so the root is step ③ or ④. judgement: at step ④, the root is the working folder. A run from `polls/` then roots ids at `polls/`, and a run from the project's folder roots them there.
- **Several projects in one repository.** From LangChain's root, step ③ takes all 624 test files of 22 projects in one run [run, 4.4]. judgement: those projects pin their own dependencies, so one interpreter may not import them all. This is unmeasured.
- **CPython.** It has no `pyproject.toml` at its root, so the root is step ③ [run].

## 6. What this file does not measure

- **purba's own lines and exit codes.** Stage one is not built. No probe ran a purba wheel. Each line about what purba would do is a judgement from the records.
- **A tester's reading.** No form was rated blind. Each candidate of section 5.1 changes a form.
- **unittest's discovery on the 18 repositories.** U is a name matcher. Only the package rule was counted, as a second matcher. No suite ran under `unittest discover`.
- **CPython's `regrtest`.** The `uv` build of CPython 3.13.13 ships no `Lib/test`, so its row is [source].
- **The run layer of the pytest suites.** The counts of `import pytest` and `conftest.py` stand for the fixtures that those suites use. I did not count the fixtures.
- **Whether LangChain's 22 projects import in one interpreter.** I did not install them.
- **The near-name matcher's predicate.** It is mine. A check of purba would need its own design, and its own count of false positives.

## Files

| path | what it holds |
|---|---|
| `research/test-layouts/repos.tsv` | the 18 repositories, each with its ref and its commit |
| `research/test-layouts/clone.sh` | the script that fetches each at its ref, and checks its commit |
| `research/test-layouts/measure.py` | the count script |
| `research/test-layouts/counts.txt` | its output for each repository, with every count of section 4 |
| `research/test-layouts/summary.tsv` | one row for each repository, with the counts of 4.1 to 4.4 and 4.6 |
| `research/test-layouts/files.tsv.gz` | one row for each P or U file: its shape, its imports and its root |
| `research/test-layouts/django/run.sh`, `out.txt` | the Django probe, and its output in this session |
| `research/test-layouts/layouts/run.sh`, `out.txt` | the probe of `tests.py`, `test.py` and doctests, and its output |
