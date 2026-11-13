# Which testing practices hold for a test written in Python, and which Python tools check each

This file answers [#955](https://github.com/kalonji-tools/purba/issues/955).
It is never merged.
It starts from the general survey, [`research/testing-practices.md` at `bbc590c`](https://github.com/kalonji-tools/purba/blob/bbc590c8ad4667f9880166b9d73ef6a6e46fbae4/research/testing-practices.md), and keeps its numbering, P1 to P14.
It does not survey the general sources again.
Every source was fetched on 2026-11-13, unless its row in the [source register](#source-register) says otherwise.

## Answer

Most of the general practices hold for Python.
What changes is the mechanism, and Python's mechanisms create failure modes that the general survey did not name.
In Python the oracle is usually the bare `assert` statement, so the language itself can disarm a test.
A test can return `False` instead of asserting it, and the runner still reports a pass.
A misspelled assertion method on a mock was, until Python 3.5, silently created by the mock and passed.
`assert (x, "message")` is a tuple, so it is always true.
And `python -O` removes every `assert`.
CPython, pytest, Hypothesis, pylint and ruff each guard one or more of these cases, at compile time, at run time or by lint.

Two of the general practices are more contested in Python than in the general survey.
P6, "no logic in a test", is contradicted by Python's own standard library documentation, which teaches a loop inside a test with `subTest`.
pytest 9.0 added the same construct to its core as `subtests`.
In the six projects I measured, between 3% and 17% of test functions contain a loop.
P4, "one condition per test", lost its premise in Python too: `subTest`, `subtests` and `parametrize` let one test function report many failures, and between 40% and 56% of test functions call more than one assertion.

The randomness dispute from the general survey has a Python answer that the sources mostly share: random input is allowed if the run can be replayed.
Hypothesis switches to derandomized runs on CI by default.
scikit-learn pins a seed of 42 for pull requests and draws a random seed from 0 to 99 only in nightly builds.
pytest-randomly reseeds `random` before each test and prints the base seed so that a failure can be repeated.

Test order is not shuffled by default in any Python runner I read.
Shuffling is a plugin (pytest-randomly) or an opt-in flag (Django's `--shuffle`), except in CPython's own CI mode, which shuffles and also fails a test file that changes process-global state.
Reruns of failed tests are a normal, documented tolerance: CPython's CI mode reruns failures, and pip reruns network tests three times on CI.

The Python linters decide syntax, as the general survey found for other languages.
They have no rule for "this test has no assertion", no rule for a branch inside a test, and no rule for a skip without a reason.
I checked the ruff 0.17.5 rule table, flake8-pytest-style's 31 rules and pylint 4.1.3's message names.
Where they do have a rule, the rule's own tracker disputes its premise: PT018 "composite assertion" (pytest already shows which side failed), S101 "assert used" (fires on every pytest test), and PLR2004 "magic value" (the Google position from the general survey says keep the literal).
Most test rules are off by default.
ruff 0.17.5 turns on six of its 31 `PT` codes by default (two of the 31 are removed). The six catch misuse, not style: a `pytest.raises` with no exception, a duplicate parametrize case, a deprecated alias, a marker with no effect, and a `pytest.warns` block with several statements.

Python adds practices the general survey did not find.
Warnings must be visible during a test run, and many projects turn them into errors (PEP 565 assigns this job to test runners).
A test must not return a value.
Exception and warning assertions must be narrow, with a `match` on the message.
A redefined test name must not silently hide the first test.
And since free-threaded CPython (PEP 703), a test should be safe to run in several threads at once.

| practice | for Python | what in Python makes the difference | who checks it, under which rule | on by default? |
|---|---|---|---|---|
| [P1 self-checking](#p1-the-test-decides-its-own-verdict) | holds; changed mechanism | bare `assert`; a returned value is ignored; mocks create any attribute; `-O` removes asserts | pytest `PytestReturnNotNoneWarning`; unittest `DeprecationWarning` (3.11); Hypothesis error; CPython `SyntaxWarning` for an assert on a tuple; ruff `B015`, `PLW0129`, `PT015`, `PT017`, `B011`; pylint `W0199`, `W0129`, `W1503` | runner checks yes; ruff `B015` and `PLW0129` yes, `PT015`, `PT017`, `B011` no; pylint yes |
| [P2 deterministic](#p2-same-code-same-result) | holds | global `random` state; CI-only derandomizing; reruns are routine | Hypothesis `FlakyFailure` and `ci` profile; pytest-randomly; pytest-rerunfailures (tolerance); CPython regrtest `--rerun` | Hypothesis yes; plugins when installed |
| [P3 independent](#p3-no-test-depends-on-another) | holds | process-global state (`sys.modules`, `os.environ`, warnings filters); default order is fixed | pytest-randomly; Django `--shuffle`; CPython regrtest `--randomize` and `--fail-env-changed`; Hypothesis `function_scoped_fixture` health check | no, except CPython's CI mode and Hypothesis |
| [P4 one condition](#p4-one-behaviour-or-condition-per-test) | changed; contested | `subTest`, `subtests` and `parametrize` keep a test going after a failure | ruff and flake8-pytest-style `PT018` only; no assertion-count rule found | no |
| [P5 clear failure output](#p5-a-failure-explains-itself) | holds; mostly met by the runner | assertion rewriting shows sub-expressions; unittest prints a test's docstring | `PT016` (`pytest.fail` with no message) | no |
| [P6 no logic](#p6-no-logic-in-a-test) | contested more than in general | the standard library teaches a loop in a test | no rule found | — |
| [P7 literal over shared](#p7-literal-and-visible-over-shared-and-dry) | contested | fixtures arrive by parameter name, from `conftest.py` or `autouse` | ruff `PLR2004`, pylint `magic-value-comparison` (an extension) | no |
| [P8 phased layout](#p8-a-phased-layout) | holds, as opinion | pytest's own docs name Arrange, Act, Assert, Cleanup | flake8-aaa `AAA01`–`AAA06` | when the plugin is installed ([B]) |
| [P9 public interface](#p9-test-through-the-public-interface) | holds, weakly | no access control, only a leading underscore | pylint `W0212` `protected-access`; ruff `SLF001` | pylint yes; ruff no |
| [P10 state over interaction](#p10-real-collaborators-and-state-over-mocks-and-interactions) | contested | `unittest.mock` is in the standard library; mocks accept any call unless specced | no rule on mock use; `PT008` is style only | — |
| [P11 fast and hermetic](#p11-fast-small-hermetic) | holds | network and timeouts are plugins | pytest-socket; pytest-timeout; ruff `ASYNC251` (async only) | ruff `ASYNC251` yes; plugins opt-in |
| [P12 adequacy](#p12-adequacy-coverage-assertions-mutants) | contested | — | coverage.py `fail_under`; mutmut | no |
| [P13 test-first](#p13-write-the-test-first) | not enforced by any Python tool | — | none | — |
| [P14 never leave a test switched off](#p14-do-not-leave-tests-switched-off) | holds; the most tooled | `xfail` passes quietly unless strict | pytest `strict_xfail` and `strict` mode; `importorskip` narrowed in 9.1 | no; opt-in |

## 1. The sources, and why I chose them

| kind | sources | why these |
|---|---|---|
| test runners and test libraries | CPython's `unittest`, `unittest.mock` and its own runner `regrtest` (R1–R4); pytest (T1); Hypothesis (T3) | `unittest` and `mock` ship with the language. pytest is the runner of four of the six projects below; CPython and Django use `unittest`. Hypothesis is the property-based library NumPy's guidelines name. |
| runner plugins | pytest-randomly, pytest-rerunfailures, pytest-timeout, pytest-socket, pytest-run-parallel (T4–T8); coverage.py, mutmut (T9, T10) | Each one encodes a position on one practice: shuffling, rerunning, time limits, network, threads, coverage, mutants. |
| linters with rules for tests | ruff (L1), flake8-pytest-style (L3), flake8-aaa (L4), pylint and pylint-pytest (L5, L6), bandit (L7) | ruff carries the flake8-pytest-style, bugbear, bandit and pylint rules in one tool with one default set. flake8-aaa is the only checker of the phased layout I found. pylint-pytest exists only to suppress false positives, which is evidence in itself. |
| warnings CPython itself gives | the compiler's assert-on-tuple warning (R6); `unittest`'s deprecations (R7); the language reference on `assert` (R5) | The issue asks for them. |
| PEPs that touch testing | PEP 230, 399, 417, 565, 654, 678, 703, and PEP 8 (checked for one claim) | Each one changes what a test runner or a test does. |
| projects with large test suites | CPython, Django, NumPy, pandas, scikit-learn, pip (X1–X6) | Two use `unittest` (CPython, Django) and four use pytest. They span a language runtime, a web framework, numeric libraries and an installer. Each has thousands of test functions (see [section 4](#4-what-the-projects-do)). |

### Liveness

Fetched with `gh api repos/<repo>/commits/<default branch> --jq .commit.committer.date` and, for the release, `gh api repos/<repo>/releases/latest` or `curl https://pypi.org/pypi/<name>/json` where the repository publishes no GitHub release.

| tool or project | default branch: last commit | latest release |
|---|---|---|
| pytest | `main`: 2026-11-11 | 9.2.0, 2026-10-27 |
| Hypothesis | `master`: 2026-11-06 | 6.170.15, 2026-11-06 |
| CPython (`unittest`, `unittest.mock`, regrtest) | `main`: 2026-11-13 | v3.15.2, tagged 2026-10-25 (v3.16.0a2 is a pre-release) |
| ruff | `main`: 2026-11-13 | 0.17.5, 2026-11-12 |
| flake8-pytest-style | `master`: 2026-11-02 | 2.2.0 (PyPI), 2025-10-20 |
| flake8-aaa | `master`: 2026-09-12 | 0.17.2 (PyPI), 2025-10-25 |
| pylint | `main`: 2026-11-10 | 4.1.3, 2026-10-11 |
| pylint-pytest | `master`: 2026-10-03 | 1.1.9.post1, 2026-09-29 |
| bandit | `main`: 2026-11-09 | 1.9.4, 2026-02-25 |
| pytest-randomly | `main`: 2026-11-08 | 5.1.0 (PyPI), 2026-11-08 |
| pytest-rerunfailures | `master`: 2026-10-07 | 16.7 (PyPI), 2026-09-17 |
| pytest-timeout | `main`: 2026-11-06 | 2.4.0 (PyPI), 2025-05-05 |
| pytest-socket | `main`: 2026-11-12 | 0.8.1, 2026-08-19 |
| pytest-run-parallel | `main`: 2026-10-31 | 0.10.0, 2026-08-04 |
| coverage.py | `main`: 2026-11-12 | 7.17.3, 2026-11-12 |
| mutmut | `main`: 2026-11-07 | 3.8.0, 2026-09-12 |
| Django | `main`: 2026-11-13 | 6.1.3 (PyPI), 2026-11-03 |
| NumPy | `main`: 2026-11-13 | 2.5.4, 2026-10-10 |
| pandas | `main`: 2026-11-12 | 3.1.1, 2026-11-04 |
| scikit-learn | `main`: 2026-11-13 | 1.9.1, 2026-09-11 |
| pip | `main`: 2026-11-07 | 26.3 (PyPI), 2026-11-02 |

Every tool and project above had a commit on its default branch in the last ten weeks.
All tool sources were read at the release tag in this table, except flake8-pytest-style and flake8-aaa, which I read at their default branch (commits `f7a3a2e` and `00264b2`).

## 2. The practices, in Python

Each practice gives the verdict for Python, the Python sources, who checks it and their known false positives, and what the six projects do.
"[A]" means fetched this session; the fetch is in the [source register](#source-register).
Project figures come from the scan in [section 4](#4-what-the-projects-do) and are syntax counts, not judgements.

### Group A. The verdict

#### P1. The test decides its own verdict

**Verdict.** Holds. Python changes how a test can fail to decide.

**What Python changes.**
- pytest uses the bare `assert` statement as its oracle and rewrites it to show values (T1 `how-to/assert.rst` [A]).
- A returned value is ignored. pytest: "Since pytest ignores return values, it might be surprising that the test will never fail based on the returned value." (T1 [A]) It emits `PytestReturnNotNoneWarning` to catch "a common mistake made by beginners who assume that returning a ``bool`` (e.g., ``True`` or ``False``) will determine whether a test passes or fails." (T1 [A])
- `unittest` deprecated the same thing in Python 3.11: "The behavior of returning a value from a test method (other than the default ``None`` value), is now deprecated." (R1 [A])
- Hypothesis is stricter: its `return_value` health check is "Deprecated; we always error if a test returns a non-None value." (T3, `_settings.py` [A])
- A mock creates any attribute that is read, so a misspelled assertion method used to pass. CPython's mock documentation: "Before 3.5, tests with a typo in the word assert would silently pass when they should raise an error." (R2 [A]) Since 3.5, a name that starts with `assert` or a common misspelling of it raises `AttributeError`, unless `unsafe=True` (R2 [A]).
- `assert (x, "msg")` is a non-empty tuple and always true. CPython's compiler warns: "assertion is always true, perhaps remove parentheses?" (R6, `Python/codegen.c` [A]).
- `-O` removes asserts: "The current code generator emits no code for an assert statement when optimization is requested at compile time." (R5 [A]) NumPy runs one CI job with `PYTHONOPTIMIZE: 2` (X3, `.github/workflows/linux.yml` [A]) and its guide says this "is not an issue when running tests with pytest" (X3 `doc/TESTS.rst` [A]).

**Who checks it.**

| check | rule | default | catches |
|---|---|---|---|
| pytest | `PytestReturnNotNoneWarning` | on | a test that returns a value |
| unittest | `DeprecationWarning` since 3.11 | shown by the unittest runner (R1 [A]) | the same |
| Hypothesis | error | on | the same, for `@given` tests |
| CPython compiler | `SyntaxWarning` "assertion is always true" | on | `assert` on a tuple |
| ruff | `B015` useless-comparison | on (L1 `DEFAULT_SELECTORS` [A]) | `x == y` with the `assert` forgotten |
| ruff | `PLW0129` assert-on-string-literal | on | an assert that always passes |
| ruff | `PT015` assert-always-false, `PT017` assert-in-except, `B011` assert-false | off | an assert in an `except` that never runs if nothing is raised (`PT017`); `assert False` removed by `-O` (`B011`) |
| pylint | `W0199` assert-on-tuple, `W0129` assert-on-string-literal, `W1503` redundant-unittest-assert | on (core checkers) | asserts that cannot fail |
| none found | "this test has no assertion" | — | no such rule in ruff 0.17.5's rule table, flake8-pytest-style PT001–PT031, or pylint 4.1.3's message names ([A], grep of each) |

**Known false positives and disputes.**
- `B015` is switched off for tests by pandas, with the reason "tests use comparisons but not their returned value" (X4 `pyproject.toml` [A]), and by NumPy for `test*.py` with no reason given (X3 `ruff.toml` [A]).
- `PytestReturnNotNoneWarning` was first meant to become an error. [pytest#10465](https://github.com/pytest-dev/pytest/issues/10465) objected: "making something raise an exception not because it's an error, but simply because some users may get confused by it, feels somewhat inconsistent". The changelog then states it "will stay a warning instead of becoming an error in the future" (T1 changelog [A]).
- Assertions inside helpers: in four of the six projects, between 4.4% and 19.6% of test functions contain no assertion-like call (section 4). The most common callees in those tests are helpers such as `check`, `eq`, `_test` and `expect_failure` in CPython, and `_check_plot_works` and `check_round_trip` in pandas ([A], scan). This is the same pattern that gives PMD its false positives in the general survey. pytest adds a Python cost: "asserts in supporting modules which are not themselves test modules will not be rewritten" unless registered (T1 [A]).

**Projects.** All six follow P1. NumPy prefers the plain `assert` over its own legacy `numpy.testing.assert_` (X3 [A]). Django prefers `assertIs(…, True)` over `assertTrue` "so you can check the actual boolean value, not the truthiness of the expression" (X2 coding style [A]).

#### P2. Same code, same result

**Verdict.** Holds. Python adds global random state, and a shared answer to the randomness dispute.

**Python sources.**
- pytest: "a flaky test indicates that the test relies on some system state that is not being appropriately controlled" (T1 `explanation/flaky.rst` [A]). pytest itself "is single-threaded, executing its tests always in the same thread, sequentially, never spawning any threads itself." (T1 [A])
- Hypothesis treats flakiness as fatal: "Hypothesis raises an exception when it detects flakiness." It gives the reason that "Hypothesis relies on deterministic behavior for the database to work." (T3 `tutorial/flaky.rst` [A]) It does not ask for full determinism: "Only the sequence of calls to Hypothesis APIs like ``draw`` … and the outcome of the test (pass or fail) need to be deterministic." (T3 [A])

**Random input: allowed if replayable.** The general survey left this as a disagreement between Meszaros and QuickCheck, with replay from a seed marked [B]. For Python it is [A]:
- Hypothesis's built-in `ci` profile sets `derandomize=True`, `deadline=None` and `database=None`, and is loaded automatically when a CI variable is set (T3 `_settings.py`, `settings.register_profile("ci", CI)` and `if is_in_ci(): settings.load_profile("ci")` [A]).
- NumPy: "a test that passes most of the time but fails occasionally with no code changes is not helpful. Make the random data deterministic by setting the random number seed before generating it." It names Hypothesis as the alternative (X3 `doc/TESTS.rst` [A]).
- scikit-learn's `global_random_seed` fixture: tests "accept the contract that they should deterministically pass for any seed value from 0 to 99 included." Nightly builds draw a seed; without the variable "42 is used as the global seed in a deterministic manner", so that "our friendly contributors are not the first people to encounter a seed-sensitivity regression" (X5 `doc/developers/global_configuration.rst` [A]).
- pytest-randomly resets `random.seed()` "before each test setup, run, and teardown", and prints a base seed that "can be passed in to repeat a failure caused by test ordering or random data" (T4 [A]).

**Contested: reruns.** The tolerance the general survey found at Google and Meta is routine in Python.
- pytest's flaky page lists pytest-rerunfailures as a mitigation (T1 [A]); that plugin "re-runs tests to eliminate intermittent failures" (T5 [A]).
- CPython's CI mode turns on `--rerun` (X1 `Lib/test/libregrtest/cmdline.py` [A]). A test that fails and then passes is marked failed only with `--fail-rerun` (X1 [A]).
- pip, on CI only: "# Mark network tests as flaky", then `pytest.mark.flaky(reruns=3, reruns_delay=2)` (X6 `tests/conftest.py` [A]).
- Hypothesis turns its 200 ms deadline off on CI; the deadline is a soft limit "where that would avoid flakiness due to timing variability" (T3 [A]).

**Who checks it.** Hypothesis, at run time, with `FlakyFailure` (T3 [A]). No static rule for a `sleep` in a synchronous test: ruff's `ASYNC251` covers `time.sleep` only inside `async` functions (L1 [A]).

**Projects.** A direct `sleep` call appears in 0.6% of CPython's test functions and in at most 0.1% elsewhere (section 4). CPython polls with exponential backoff instead (`support.sleeping_retry`, X1 [A]).

#### P3. No test depends on another

**Verdict.** Holds. Python adds process-global state as the main channel, and keeps a fixed order by default.

**Python sources.**
- A fresh instance per test. pytest: "each test has a unique instance of the class. Having each test share the same class instance would be very detrimental to test isolation" (T1 `getting-started.rst` [A]). `unittest`'s loader "creates an instance of the class for each test method" (R1 [A]).
- A fixed order. `unittest`: "The order in which the various tests will be run is determined by sorting the test method names" (R1 [A]). pytest's changelog: "Tests are now ordered by definition order in more cases." (T1 [A]) A request to randomize order in pytest's core, [pytest#598](https://github.com/pytest-dev/pytest/issues/598), was closed after a commenter pointed to an existing plugin (T2 [A]).
- Shuffling is opt-in. Django: `--shuffle` "can help detect tests that aren't properly isolated", and the order "is a deterministic function of the integer seed given" (X2 `docs/ref/django-admin.txt` [A]). Django's own GitHub workflows run `runtests.py` without `--shuffle` ([A], grep of `.github/workflows`).
- CPython's CI mode is the exception. `--fast-ci` and `--slow-ci` set `randomize = True` and `fail_env_changed = True` (X1 `cmdline.py` [A]). The Makefile and the GitHub workflows use `--fast-ci` (X1 [A]).

**Python-specific detector.** CPython's `--fail-env-changed` fails a test file that "alters the environment" (X1 [A]). It compares process-global resources before and after each file, among them `sys.argv`, `cwd`, `os.environ`, `sys.path`, `warnings.filters`, `logging._handlers`, `threading._dangling` and open files (X1 `save_env.py` [A]). This is a dynamic check of the state that the general survey's order-dependence studies blame.

**Shared fixtures for cost.** pytest's fixture scopes exist "to only be invoked once per test *module* … thus saving time" (T1 `how-to/fixtures.rst` [A]); the default is once per test function. Django reorders tests by class so that "all ``TestCase`` code starts with a clean database", and warns that this "may reveal unexpected dependencies on test case ordering" (X2 `docs/topics/testing/overview.txt` [A]). Hypothesis has a `function_scoped_fixture` health check because a function-scoped fixture runs "once for the whole function, not once per test case" (T3 [A]).

**Who checks it.** All dynamic: pytest-randomly (on when installed), Django `--shuffle`, CPython `--randomize` and `--fail-env-changed`, Hypothesis's health check (on). No static rule found.

#### Group B. The diagnosis

#### P4. One behaviour or condition per test

**Verdict.** Changed, and contested. Python removed the general survey's main reason for it.

The general survey's reason was Meszaros's: "when one assertion fails, the rest of the test is not executed."
A plain `assert` still stops a Python test.
But both runners now offer a keep-going construct inside one test function:
- `unittest.TestCase.subTest`, since 3.4: "Without using a subtest, execution would stop after the first failure, and the error would be less easy to diagnose" (R1 [A]).
- pytest `subtests`, since 9.0 and marked experimental: "Each assertion failure or error is caught by the context manager and reported individually" (T1 `how-to/subtests.rst` [A]).
- `pytest.mark.parametrize` turns one function into many test items (T1 [A]).

**Who checks it.** Only `PT018` composite-assertion (ruff: off, in its "pedantic" category; flake8-pytest-style: on when installed, [B]). Its stated reason is that a composite assertion's "failure message will not indicate which condition failed" (L1 [A]). [ruff#20779](https://github.com/astral-sh/ruff/issues/20779), open, disputes that for pytest, because pytest shows `assert (True and False)` (L2 [A]). I found no rule that counts assertions per test in ruff, flake8-pytest-style or pylint ([A], grep of rule names; pylint's `too-many-*` messages do not include assertions).

**Projects.** Between 40.3% (Django) and 56.4% (scikit-learn) of test functions make more than one assertion-like call. pandas lists `PT018` among rules it has "intentionally disabled", with no reason given for that rule (X4 [A]).

#### P5. A failure explains itself

**Verdict.** Holds. In Python the runner supplies most of it, which is why hand-written messages are rare.

**Python sources.**
- The language reference: "it is unnecessary to include the source code for the expression that failed in the error message; it will be displayed as part of the stack trace." (R5 [A])
- pytest shows "the values of the most common subexpressions including calls, attributes, comparisons, and binary and unary operators" (T1 [A]).
- `unittest`'s `assertEqual` calls a type-specific function "to generate a more useful default error message", and `longMessage` defaults to `True`, so a custom message is appended rather than replacing it (R1 [A]).
- PEP 678 (exception notes, Final, 3.11) names testing libraries as its first use case: they "may wish to show the values involved in a failing assertion, or the steps to reproduce a failure". It names pytest and Hypothesis (PEP 678 [A]). pytest's `match` checks those notes too (T1 [A]).

**Names and docstrings: the projects disagree.** `unittest` prints a test's docstring line beside its name (R1, the `subTest` example output [A]).
- NumPy: "``test_`` functions or methods should not have a docstring, because that makes it hard to identify the test from the output" (X3 [A]).
- Django: "In test docstrings, state the expected behavior that each test demonstrates." (X2 coding style [A])

**Who checks it.** `PT016` (`pytest.fail` with no message; ruff off). pytest `strict_parametrization_ids` errors on duplicate parameter IDs (off; part of `strict` mode) (T1 [A]).

**Projects.** Of all `assert` statements, the share with a message is 0.4% (pandas) to 10.3% (pip). Django has one `assert` statement in its tests and uses `unittest` methods (section 4).

### Group C. Reading

#### P6. No logic in a test

**Verdict.** Contested, more than in the general survey, because Python's own documentation teaches the opposite.

- The `unittest` documentation's `subTest` example is a `for` loop inside a test method (R1 [A]).
- pytest's `subtests` example is a `for` loop inside a test function (T1 [A]).
- `parametrize` moves the loop out of the function, as Meszaros's Parameterized Test does (T1 [A]).
- Hypothesis property tests compute expected relations by design (T3 [A]).

**Who checks it.** No Python rule found for a branch or a loop in a test (ruff 0.17.5 rule table and flake8-pytest-style PT001–PT031, [A] grep). The nearest is `PT012`: a `pytest.raises` block "should only contain a single simple statement", which allows "`for` loops with empty bodies" (L1 [A]).

**Projects.** Loops in 3.2% (pip) to 17.2% (scikit-learn) of test functions; branches in 3.7% (Django) to 16.0% (scikit-learn) (section 4).

#### P7. Literal and visible over shared and DRY

**Verdict.** Contested. Python adds an implicit channel for shared setup: fixtures that arrive by parameter name.

- pytest fixtures are found by name, from the test module, `conftest.py` files or plugins. An `autouse` fixture applies to tests that never name it: in pytest's own example, "both tests are affected by it, even though neither test requested it" (bold markup dropped) (T1 `how-to/fixtures.rst` [A]).
- NumPy: "fixtures should be used sparingly". It encourages "setup and teardown methods that are called explicitly by the tests that need them", because that "is thread-safe" (X3 [A]).
- Magic values: ruff `PLR2004` and pylint's optional `magic-value-comparison` extension flag literals in comparisons. ruff's documentation says such values "are discouraged by [PEP 8]" (L1 [A]). I searched PEP 8 for "magic" and found it only in the sense of dunder names; I found no passage that discourages literal values ([A], `grep -n -i magic pep-0008.rst`).

**Known false positives.**
- Fixture parameters collide with general rules. pylint documents it on its own `redefined-outer-name` page: "A common issue is that this message is triggered when using `pytest` fixtures" (L5 [A]). pylint-pytest exists "to suppress pytest-related false positives" in `unused-argument`, `unused-import` and `redefined-outer-name` (L6 [A]).
- [ruff#4368](https://github.com/astral-sh/ruff/issues/4368), open, asks to turn off `ARG` (unused arguments, because "fixtures nevertheless are functionally relevant") and lists `PLR2004` among rules it calls debatable in tests (L2 [A]).

**Projects.** pandas has `PLR2004` under "TODO: Enable gradually" (X4 [A]). None of the six enables it for tests ([A], grep of their lint configs).

#### P8. A phased layout

**Verdict.** Holds, as opinion. pytest states it in its own documentation.

pytest's "Anatomy of a test" breaks a test into four steps: Arrange, Act, Assert and Cleanup. "**Act** is the singular, state-changing action that kicks off the **behavior** we want to test." (T1 `explanation/anatomy.rst` [A]). Its fixture page says fixtures "define the steps and data that constitute the *arrange* phase" (T1 [A]).

**Who checks it.** flake8-aaa: `AAA01` (no Act block), `AAA02` (more than one Act block), `AAA03`–`AAA05` (blank lines between blocks), `AAA06` (comment in the Act block) (L4 [A]). It finds the blocks by blank lines and by a line like `result =`. Whether flake8 enables its codes by default once installed: [B], not checked in flake8's source.

**Projects.** None of the six uses flake8-aaa ([A], grep of lint configs). I did not measure layout.

### Group D. Coupling

#### P9. Test through the public interface

**Verdict.** Holds, weakly. Python has no access control; privacy is a leading underscore.

- pylint `W0212` `protected-access` is on by default (L5 [A]); ruff `SLF001` is off. Their trackers' false positives concern access between instances of one class, not tests ([ruff#9022](https://github.com/astral-sh/ruff/issues/9022) and others, titles only [A]).
- CPython tests some internals on purpose. PEP 399 requires that C accelerator code "**must** pass the test suite used for the pure Python code", and that "a module must be tested both with and without its accelerated code" (PEP 399 [A]).
- One test file per module, which the general survey says Google warns against: NumPy asks that "Every Python module, extension module, or subpackage … should have a corresponding ``test_<name>.py`` file" (X3 [A]).
- Testing what is installed: pytest recommends the `importlib` import mode for new projects and "**strongly**" suggests a `src` layout, so that tests "can run against an installed version" (T1 `explanation/goodpractices.rst` [A]).

**Projects.** Between 3.5% (NumPy) and 7.3% (pandas) of test functions read an underscore attribute on something other than `self` or `cls` (section 4).

#### P10. Real collaborators and state over mocks and interactions

**Verdict.** Contested, as in general. Python's mock adds its own hazard, and the projects mock little.

- `unittest.mock` has been in the standard library since 3.3 (PEP 417 [A]).
- CPython's mock documentation names the classicist's worry: after a rename, "tests can all pass even though your code is broken", and "this is another reason why you need integration tests as well as unit tests" (R2 [A]).
- Autospeccing fixes the call signature, but it "is not the default behaviour", because introspection can "trigger code execution" (R2 [A]).
- Python patches names, not types: "you patch where an object is *looked up*, which is not necessarily the same place as where it is defined" (R2 [A]).

**Who checks it.** No rule on how much a test mocks. `PT008` (patch with a lambda) is style. No tool for "mock only types you own" ([C], none found).

**Projects.** A mock, `patch`, `mocker` or `monkeypatch` appears in 0.5% (NumPy) to 13.0% (pip) of test functions (section 4). By this measure the six projects mostly test real collaborators.

### Group E. Cost to run

#### P11. Fast, small, hermetic

**Verdict.** Holds. Python enforces it with plugins and markers, not with sizes.

- pytest-socket "Disables all network calls flowing through Python's `socket` interface" (T7 [A]). pip runs with `--disable-socket`, `--allow-unix-socket`, `--allow-hosts=localhost` (X6 `pyproject.toml` [A]).
- pytest-timeout: "By default the plugin will not time out any tests" (T6 [A]). scikit-learn uses pytest's `faulthandler_timeout = 600` to dump tracebacks "if a test hangs for more than 10 min (likely due to a deadlock)" (X5 [A]).
- CPython's regrtest runs "special resource intensive tests" only when named with `-u` (X1 `cmdline.py` [A]). An internet timeout "should not mark a test as failed, but skip the test instead" (X1 `Lib/test/support/__init__.py` [A]).
- NumPy labels slow tests with `pytest.mark.slow` (X3 [A]); pip has a `network` marker (X6 [A]).

**Who checks it.** ruff `ASYNC251` (on; async only). The rest is dynamic and opt-in.

### Group F. Adequacy

#### P12. Adequacy: coverage, assertions, mutants

**Verdict.** Contested, and no project I read gates on a coverage number.

- coverage.py measures branch coverage only on request: `[run] branch` is "(boolean, default False)" (T9 [A]). `fail_under` is optional (T9 [A]).
- PEP 399 is the one Python source that makes coverage a precondition: "the test suite for the module must have comprehensive coverage of the pure Python implementation before the acceleration code may be added" (PEP 399 [A]).
- scikit-learn's contributor guide is coverage-driven: "Find a low hanging fruit, looking at which lines are not tested, write or adapt a test specifically for these lines." (X5 [A])

**Projects.**

| project | coverage gate | source |
|---|---|---|
| scikit-learn | a ratchet: project coverage may not drop by more than 1% | `.codecov.yml` [A] |
| NumPy | informational only | `.codecov.yml` [A] |
| pandas | statuses off | `codecov.yml` [A] |
| Django | `diff-cover … --fail-under=0`, a report | `.github/workflows/coverage_tests.yml` [A] |
| pip, CPython | none found | grep of configs [A] |

None of the six configures mutmut or another mutation tool ([A], grep of `pyproject.toml`, workflows, `noxfile.py`, `tox.ini`).

### Group G. Process

#### P13. Write the test first

**Verdict.** No Python tool checks it, and no Python source I read requires it.

- Django's tutorial presents test-driven development as one approach, then: "More often, a newcomer to testing will create some code and later decide that it should have some tests." (X2 `docs/intro/tutorial05.txt` [A])
- Django's review checklist asks: "Is there a proper regression test (the test should fail before the fix is applied)?" (X2 [A]) That checks that the test can fail, not when it was written.
- NumPy: "please write the tests as you develop your code" (X3 [A]).

#### P14. Do not leave tests switched off

**Verdict.** Holds, and it is the practice Python tools support most.

- pytest's `xfail` has two documented uses that pull against each other. The reference says non-strict xfail "is particularly useful to mark *flaky* tests (tests that fail at random) to be tackled later" (T1 [A]). The flaky-tests page calls it "a manual quarantine, and is rather dangerous to use permanently" (T1 [A]).
- `strict_xfail` makes an unexpected pass fail the suite; it defaults to `false`. pytest 9.0 added a `strict` option that enables it with `strict_config`, `strict_markers` and `strict_parametrization_ids`; it also defaults to `false` (T1 [A]). The idea goes back to [pytest#814](https://github.com/pytest-dev/pytest/issues/814), a debate on separating "expected to fail" from "flaky" (T2 [A]).
- pytest 9.1 narrowed `importorskip`: it used to skip on any `ImportError`, so a broken install "would still silently skip the test"; now it catches only `ModuleNotFoundError` by default (T1 `deprecations.rst` [A]).
- `unittest`'s skip decorators take a required `reason` argument (R1 [A]).

**Who checks it.** pytest `strict_xfail` (off), `--strict-markers` (off; a misspelled marker otherwise creates a new one), and the `-r` summary flags. No ruff, flake8-pytest-style or pylint rule flags a skip without a reason ([A], grep of rule names).

**Projects.** pandas and pip set `xfail_strict = true` (X4, X6 [A]). pandas bans the imperative call: "Use pytest.mark.xfail instead of pytest.xfail" (X4 [A]). NumPy and pandas run with `--strict-markers --strict-config` (X3, X4 [A]). Every function-level skip decorator found in five projects carries a reason; in pip, 58 of 87 do (section 4). For the two `unittest` projects this is structural, since the decorators require the argument.

## 3. What Python adds

These practices appear in Python's own sources but not in the general survey.

| id | practice | Python sources | who checks it | contested? |
|---|---|---|---|---|
| PY1 | Show warnings during a test run, and turn them into errors | PEP 565 (Final, 3.7): "Developers of test runners are advised to implement logic equivalent to" `warnings.simplefilter("default")`. The `unittest` runner shows `DeprecationWarning`, `PendingDeprecationWarning`, `ResourceWarning` and `ImportWarning` by default (R1 [A]). pytest shows `DeprecationWarning` "as recommended by :pep:`565`" (T1 [A]). | pytest `filterwarnings = error`: NumPy and pandas set it (X3, X4 [A]); pip makes its own deprecation warning an error (X6 [A]); Django's CI runs `python -Wall` (X2 [A]) | yes: scikit-learn runs with `--disable-pytest-warnings` (X5 [A]) |
| PY2 | A test never returns a value | see [P1](#p1-the-test-decides-its-own-verdict) | pytest warning; `unittest` deprecation; Hypothesis error | the strength: pytest#10465 kept it a warning |
| PY3 | Do not rely on `assert` outside the test runner's control | `-O` removes asserts (R5 [A]); pytest rewrites only test modules (T1 [A]) | bandit `B101`, ruff `S101` (off), ruff `B011` (off) | yes: see below |
| PY4 | Spec a mock to the real object | autospec makes a wrong call raise `TypeError` (R2 [A]) | the runtime, when `autospec=True` or `spec=` is passed; misspelled `assert` names raise since 3.5 | autospec is off by default, on purpose (R2 [A]) |
| PY5 | Check an exception or a warning narrowly: one statement, and a `match` on the message | NumPy: a `match` "should always be used to precisely target the intended warning or error" (X3 [A]). Django: use `assertRaisesMessage` "so you can check the exception or warning message" (X2 [A]). | ruff `B017` (on), `PT010` (on), `PT031` (on), `PT011`, `PT012`, `PT027`, `PT030` (off) | yes: see below |
| PY6 | No test silently lost to a name collision or a typo | CPython's test lint selects `F811` as "useful for finding test methods with the same name" (X1 `Lib/test/.ruff.toml` [A]) | ruff `F811` (on), pylint `E0102` `function-redefined`; `PT014` duplicate parametrize cases (on); `PT028` a test parameter with a default "will always have the default value" (off); pytest `strict_markers` (off); pytest exit code 5 "No tests were collected" and `unittest` exit code 5 when "no tests were run or skipped" (T1, R1 [A]) | no dispute found |
| PY7 | A test is safe to run in several threads at once | PEP 703 (Final, 3.13) makes the GIL optional. pytest-run-parallel exists "to discover thread-safety issues", which "is of vital importance after PEP703" (T8 [A]). NumPy's guide has a section "Writing thread-safe tests" (X3 [A]). scikit-learn lists `thread_unsafe_fixtures` (X5 [A]). | pytest-run-parallel (opt-in) | new; only the numeric projects address it |
| PY8 | Surface exceptions that cannot propagate | pytest: exceptions in `__del__` and unhandled thread exceptions "may go unnoticed because they don't cause the program itself to crash" (T1 `how-to/failures.rst` [A]) | pytest `PytestUnraisableExceptionWarning`, `PytestUnhandledThreadExceptionWarning` (on, as warnings); pandas makes them errors (X4 [A]) | no dispute found |

**PY3 in detail.** Bandit's `B101` exists because projects "used assert to enforce interface constraints", which `-O` removes (L7 [A]). In a pytest suite every test assertion is an `assert`, so the rule fires on every test. Bandit's own documentation suggests a `skips` pattern "when you use assert statements in test cases" (L7 [A]). ruff's tracker holds the same complaint: [ruff#4368](https://github.com/astral-sh/ruff/issues/4368) (open) and [ruff#26570](https://github.com/astral-sh/ruff/issues/26570) ("S101 should not show for test files", closed as a duplicate of #4368) (L2 [A]).

**PY5 false positives** (L2, L3 [A]; titles fetched, bodies read where quoted):
- `PT011` too broad: [ruff#17026](https://github.com/astral-sh/ruff/issues/17026) "False positive PT011 when using pytest.raises as a function" (closed); [ruff#5157](https://github.com/astral-sh/ruff/issues/5157) (open) asks to raise it "only if exception info is unused", because a later check on `exc.value` is "plenty specific enough". The default list of exceptions that need a `match` is `BaseException`, `Exception`, `ValueError`, `OSError`, `IOError`, `EnvironmentError`, `socket.error` (L1 `options.rs` [A]).
- `PT012` one statement: [ruff#9730](https://github.com/astral-sh/ruff/issues/9730) "False Positive PT012 for async iterator" (closed); [ruff#19363](https://github.com/astral-sh/ruff/issues/19363) "PT012: and generators" (open); [flake8-pytest-style#93](https://github.com/m-burst/flake8-pytest-style/issues/93) "PT012: False positive on context manager with statement" (closed); [flake8-pytest-style#114](https://github.com/m-burst/flake8-pytest-style/issues/114) "PT012 possible false positive: context manager that catches exceptions" (open).
- pandas switches off `PT012` and `PT017` as "Intentionally disabled", with no per-rule reason (X4 [A]).

## 3b. Practices Python held and later dropped or softened

| practice | what replaced it | what moved it | sources |
|---|---|---|---|
| `yield`-generated tests | `pytest.mark.parametrize` | "This form of test function doesn't support fixtures properly". They xfail since pytest 4.0 and are a collection error since 8.4. | T1 `deprecations.rst` [A] |
| Running nose-style tests in pytest | native `setup_method`, fixtures | "``nose`` has been in maintenance mode-only for years". Deprecated in 7.2, removed in 8.0. | T1 [A] |
| `unittest` alias methods (`failUnless`, `assertEquals` and others) | `assertTrue`, `assertEqual` and others | Deprecated between 3.1 and 3.5, removed in 3.12. | R7 `whatsnew/3.12.rst` [A] |
| Returning a value from a test | an `assert` | Moved in opposite directions: pytest planned an error and kept a warning (pytest#10465); `unittest` deprecated it in 3.11; Hypothesis made it always an error. | T1, R1, T3 [A] |
| A leading underscore on a fixture that returns nothing (`PT004`, `PT005`) | no convention | ruff removed both in 0.8.0: "marking fixtures that do not return a value with an underscore isn't a practice recommended by the pytest community". flake8-pytest-style still lists them. | L1, L3 [A] |
| `pytest.importorskip` skipping on any `ImportError` | skip only on `ModuleNotFoundError` | A broken install was hidden as a skip. Removed in 9.1. | T1 [A] |
| A mock that accepts misspelled assertion methods | `AttributeError` on such names, since 3.5 | Tests "would silently pass when they should raise an error". | R2 [A] |
| `pytest.yield_fixture` | `pytest.fixture` | A deprecated alias, to be removed in pytest 10. ruff `PT020` is on by default. | T1, L1 [A] |
| NumPy: xunit-style `setup`/`teardown` and `numpy.testing.assert_raises` | explicit setup calls; `pytest.raises` with `match` | Thread safety under `pytest-run-parallel`; better failure output from rewritten asserts. | X3 [A] |
| Non-strict `xfail` as a flaky-test marker | strict `xfail`, opt-in | pytest's own docs now call it "rather dangerous to use permanently", but the reference still recommends it for flaky tests, and the default stays non-strict. Softened, not dropped. | T1 [A] |

## 4. What the projects do

I read each project's contributor guide and its test, lint and CI configuration at the release tag.
I then scanned every test function with a script that walks Python's syntax tree.
The script is in the [appendix](#appendix-the-scan-script).

A test function is a `def test_*` at module level, or a method whose name starts with `test` inside a class, in a file named `test_*.py` or `*_test.py`.
Nested functions and classes inside a test are not counted as part of it.
"Assertion-like call" means an `assert` statement, or a call whose name starts with `assert`, or a call named `raises`, `warns`, `fail`, `approx` or `deprecated_call`.
These are syntax proxies.
They miss assertions in helpers, and they count a loop that only builds data.

Command: `nix-shell -p python3 --run 'python3 -I scan_tests.py <root>'` (Python 3.13.15), on `Lib/test` (CPython), `tests` (Django), `numpy` (NumPy), `pandas/tests` (pandas), `sklearn` (scikit-learn) and `tests` (pip).

| measure (share of test functions) | CPython v3.15.2 | Django 6.1.3 | NumPy 2.5.4 | pandas 3.1.1 | scikit-learn 1.9.1 | pip 26.3 |
|---|---|---|---|---|---|---|
| test files / test functions | 828 / 30,120 | 627 / 9,760 | 180 / 7,208 | 981 / 21,773 | 259 / 5,390 | 147 / 2,065 |
| P1: no assertion-like call | 19.6% | 4.4% | 7.6% | 5.0% | 7.2% | 8.4% |
| P4: more than one assertion-like call | 51.1% | 40.3% | 56.0% | 45.1% | 56.1% | 46.8% |
| P4: uses `subTest` or `subtests` | 4.5% | 6.9% | 0 | 0 | 0 | 0 |
| P5: `assert` statements with a message | 25 of 512 | 0 of 1 | 84 of 2,984 | 69 of 18,408 | 192 of 7,163 | 355 of 3,444 |
| P5: has a docstring | 5.5% | 19.0% | 5.5% | 0.6% | 28.3% | 46.5% |
| P6: has a loop | 15.1% | 9.2% | 15.3% | 4.4% | 17.2% | 3.2% |
| P6: has a branch (`if`, conditional expression, `match`) | 6.7% | 3.7% | 7.7% | 13.6% | 16.0% | 7.2% |
| P9: reads an underscore attribute (not on `self`, `cls`, `sys`, `os`) | 6.1% | 6.1% | 3.5% | 7.3% | 4.0% | 7.0% |
| P10: uses mock, `patch`, `mocker` or `monkeypatch` | 2.0% | 2.1% | 0.5% | 0.9% | 1.2% | 13.0% |
| P11: calls `sleep` directly | 0.6% | 0.1% | 0 | 1 test | 1 test | 0 |
| P14: function-level skip decorators, with a reason | 1,662 / 1,662 | 78 / 78 | 225 / 225 | 119 / 119 | 15 / 15 | 58 / 87 |
| P14: function-level `xfail` / `expectedFailure` decorators | 11 | 5 | 37 | 133 | 12 | 12 |
| `parametrize` decorators | 0 | 0 | 1,301 | 7,858 | 3,475 | 452 |

What each project says about its choices, where it says anything:

| project | practice | what it does | does it say why? |
|---|---|---|---|
| CPython | P2, P3 | CI mode shuffles, reruns failures, and fails on changed environment | the option help texts only (X1 [A]) |
| CPython | PY6 | lints tests with `F811` | yes: "useful for finding test methods with the same name" (X1 [A]) |
| Django | P3 | `--shuffle` available, not used in its own CI | the option's purpose, not why CI omits it (X2 [A]) |
| Django | P5 | docstrings that state expected behaviour; `assertRaisesMessage` | yes (X2 [A]) |
| NumPy | P5 | no docstrings in tests | yes: they hide the test's identity in verbose output (X3 [A]) |
| NumPy | P7, PY7 | fixtures sparingly; explicit setup calls | yes: thread safety (X3 [A]) |
| NumPy | P2 | seed random data, or use Hypothesis | yes (X3 [A]) |
| NumPy | P1, PY3 | `B011` (`assert False`) ignored; one CI job with `PYTHONOPTIMIZE: 2` | no reason given in the config (X3 [A]) |
| pandas | P1 | `B015` off | yes: "tests use comparisons but not their returned value" (X4 [A]) |
| pandas | P4, PY5 | `PT012`, `PT017`, `PT018` off | only "Intentionally disabled" (X4 [A]) |
| pandas | P7 | `PLR2004` off | "TODO: Enable gradually" (X4 [A]) |
| pandas | P14, PY1, PY8 | `xfail_strict`, strict markers, warnings and unraisable exceptions as errors | no |
| scikit-learn | P2 | seed contract 0–99, 42 by default | yes, at length (X5 [A]) |
| scikit-learn | PY1 | `--disable-pytest-warnings` | no |
| scikit-learn | P12 | coverage may drop at most 1% | yes: to "limit noisy red coverage status on github PRs" (X5 [A]) |
| pip | P11 | sockets disabled except localhost | no |
| pip | P2 | network tests rerun 3 times on CI | the comment "Mark network tests as flaky" (X6 [A]) |
| pip | P14 | `xfail_strict = true` | no |

## [C] rows and what I could not do

| item | why |
|---|---|
| A tool that checks "mock only types you own" in Python | Not found. |
| Whether flake8 enables a plugin's codes by default once installed (flake8-aaa, flake8-pytest-style) | Not checked in flake8's source; [B]. |
| pylint tracker false positives for `protected-access` in tests | My search found none specific to tests (one search hit the GitHub search rate limit); not established. |
| Why pytest keeps `strict_xfail` off by default | Not stated in the documentation or the issues I read. pytest#7503 discusses what `--strict` should imply, not the default. |
| Why Django's CI does not pass `--shuffle` | Not stated in what I read. |
| pip and CPython coverage gates | None found in the configuration files I grepped; a gate elsewhere (for example a hosted service setting) would not show. |
| Whether a project's tests actually run under `-O` beyond NumPy's one job | Not checked. |
| Assertions in helpers, as a share of all checks | The scan cannot follow calls; the "no assertion-like call" row overstates tests that check nothing. |
| The CPython developer guide (a separate repository) | Not read; CPython rows rely on the runner's source and `Lib/test` configuration. |
| Whether "PEP 8 discourages magic values", as ruff's `PLR2004` page says | I found no such passage in PEP 8; the claim is ruff's. |

## Source register

Shell commands: `git clone --depth 1 --branch <tag> <repo>` (sparse where marked), then `grep` and `sed` on the files named.
`curl -sSfL https://raw.githubusercontent.com/<repo>/<tag>/<path>` for single files.
GitHub issue titles came from `gh search issues --repo <repo> "<query>" --json number,title,state`; issue bodies quoted came from `gh issue view <n> -R <repo> --json title,state,body,comments`.

| id | source | fetched from | tier |
|---|---|---|---|
| R1 | CPython `Doc/library/unittest.rst` | `python/cpython` tag `v3.15.2` (commit `5e58eaa`), sparse clone | [A] |
| R2 | CPython `Doc/library/unittest.mock.rst` | same | [A] |
| R3 | CPython regrtest: `Lib/test/libregrtest/cmdline.py`, `save_env.py` | `python/cpython` tag `v3.15.2`, full clone | [A] |
| R4 | CPython `Lib/test/support/__init__.py`; `Lib/test/.ruff.toml`; `Makefile.pre.in`; `.github/workflows/*.yml` | same | [A] |
| R5 | Python language reference, "The assert statement" (`Doc/reference/simple_stmts.rst`) | https://raw.githubusercontent.com/python/cpython/v3.15.2/Doc/reference/simple_stmts.rst | [A] |
| R6 | CPython `Python/codegen.c` (`codegen_assert`), `Python/compile.c` (`_PyCompile_Warn` emits a `SyntaxWarning`) | `python/cpython` tag `v3.15.2` | [A] |
| R7 | CPython `Doc/whatsnew/3.11.rst`, `3.12.rst` | same | [A] |
| T1 | pytest 9.2.0 docs: `how-to/assert.rst`, `how-to/fixtures.rst`, `how-to/subtests.rst`, `how-to/skipping.rst`, `how-to/capture-warnings.rst`, `how-to/failures.rst`, `explanation/flaky.rst`, `explanation/anatomy.rst`, `explanation/fixtures.rst`, `explanation/goodpractices.rst`, `getting-started.rst`, `reference/reference.rst`, `reference/exit-codes.rst`, `deprecations.rst`, `changelog.rst` | `pytest-dev/pytest` tag `9.2.0` (commit `ae1d0ff`) | [A] |
| T2 | pytest issues #598, #814, #7503, #10465; #13477 (from the changelog text) | `gh issue view` | [A] |
| T3 | Hypothesis 6.170.15: `docs/tutorial/flaky.rst`, `docs/tutorial/settings.rst`, `src/hypothesis/_settings.py` | `HypothesisWorks/hypothesis` tag `v6.170.15` (commit `9ab0be6`), sparse clone | [A] |
| T4 | pytest-randomly README | https://raw.githubusercontent.com/pytest-dev/pytest-randomly/5.1.0/README.rst | [A] |
| T5 | pytest-rerunfailures README | https://raw.githubusercontent.com/pytest-dev/pytest-rerunfailures/16.7/README.rst | [A] |
| T6 | pytest-timeout README | https://raw.githubusercontent.com/pytest-dev/pytest-timeout/2.4.0/README.rst | [A] |
| T7 | pytest-socket README | https://raw.githubusercontent.com/miketheman/pytest-socket/0.8.1/README.md | [A] |
| T8 | pytest-run-parallel README | https://raw.githubusercontent.com/Quansight-Labs/pytest-run-parallel/v0.10.0/README.md | [A] |
| T9 | coverage.py `doc/config.rst`, `doc/branch.rst` | https://raw.githubusercontent.com/nedbat/coveragepy/7.17.3/doc/config.rst | [A] |
| T10 | mutmut README | https://raw.githubusercontent.com/boxed/mutmut/3.8.0/README.rst | [A], liveness and purpose only |
| L1 | ruff 0.17.5: `crates/ruff_linter/src/codes.rs`, `settings/mod.rs` (`DEFAULT_SELECTORS`, 407 rules), rule doc comments under `crates/ruff_linter/src/rules/`, `crates/ruff_workspace/src/options.rs`, `docs/tutorial.md` | `astral-sh/ruff` tag `0.17.5` (commit `c91de8a`), sparse clone | [A] |
| L2 | ruff issues #4368, #5157, #9022, #9730, #17026, #19363, #20779, #26570 | `gh search issues`, `gh issue view` | [A]; #9022 title only |
| L3 | flake8-pytest-style README and `docs/rules/PT004.md`; issues #93, #114 | `m-burst/flake8-pytest-style` commit `f7a3a2e`; `gh search issues`, `gh issue view 114` | [A]; #93 title only |
| L4 | flake8-aaa README and `docs/error_codes/` | `jamescooke/flake8-aaa` commit `00264b2` | [A] |
| L5 | pylint 4.1.3: `pylint/checkers/`, `pylint/extensions/magic_value.py`, `doc/data/messages/`, `doc/user_guide/checkers/extensions.rst` | `pylint-dev/pylint` tag `v4.1.3` (commit `e75de10`) | [A] |
| L6 | pylint-pytest README | https://raw.githubusercontent.com/pylint-dev/pylint-pytest/v1.1.9.post1/README.md | [A] |
| L7 | bandit `bandit/plugins/asserts.py` (B101) | https://raw.githubusercontent.com/PyCQA/bandit/1.9.4/bandit/plugins/asserts.py | [A] |
| PEP | PEP 8, 230, 399, 417, 565, 654, 678, 703 | https://raw.githubusercontent.com/python/peps/main/peps/pep-NNNN.rst at `python/peps` `main` = `155e5be` | [A] |
| X1 | CPython, see R3, R4 | tag `v3.15.2` | [A] |
| X2 | Django: `docs/topics/testing/overview.txt`, `docs/ref/django-admin.txt`, `docs/internals/contributing/writing-code/coding-style.txt`, `…/submitting-patches.txt`, `docs/intro/tutorial05.txt`, `.github/workflows/*.yml`, `tox.ini` | `django/django` tag `6.1.3` (commit `f16e25d`) | [A] |
| X3 | NumPy: `doc/TESTS.rst`, `pytest.ini`, `ruff.toml`, `.codecov.yml`, `.github/workflows/linux.yml` | `numpy/numpy` tag `v2.5.4` (commit `a3d0ef1`) | [A] |
| X4 | pandas: `pyproject.toml` (ruff and pytest sections), `codecov.yml` | `pandas-dev/pandas` tag `v3.1.1` (commit `3dcc6fb`) | [A] |
| X5 | scikit-learn: `pyproject.toml`, `.codecov.yml`, `doc/developers/contributing.rst`, `doc/developers/global_configuration.rst` | `scikit-learn/scikit-learn` tag `1.9.1` (commit `866c0f5`) | [A] |
| X6 | pip: `pyproject.toml`, `tests/conftest.py` | `pypa/pip` tag `26.3` (commit `191854f`) | [A] |

## Appendix: the scan script

Run as `python3 -I scan_tests.py <root>`.
It reads files only; it imports nothing from the projects.

<details>
<summary>scan_tests.py</summary>

```python
"""Count syntax proxies for testing practices in a project's test functions.

Usage: python -I scan_tests.py <project-root>
A test function is a `def test_*` at module level, or a method whose name
starts with `test` in a class, inside a file named test_*.py or *_test.py.
Proxies are syntax only.
"""
import ast
import os
import sys

ASSERT_PREFIXES = ("assert", "assert_")
MOCK_NAMES = {"mock", "patch", "Mock", "MagicMock", "AsyncMock", "create_autospec", "mocker"}


def callee_name(call):
    f = call.func
    if isinstance(f, ast.Attribute):
        return f.attr
    if isinstance(f, ast.Name):
        return f.id
    return ""


def is_assert_call(call):
    n = callee_name(call)
    return n.startswith(ASSERT_PREFIXES) or n in {"raises", "warns", "fail", "approx", "deprecated_call"}


def deco_names(fn):
    out = []
    for d in fn.decorator_list:
        node = d.func if isinstance(d, ast.Call) else d
        name = node.attr if isinstance(node, ast.Attribute) else getattr(node, "id", "")
        has_reason = isinstance(d, ast.Call) and (
            any(k.arg == "reason" for k in d.keywords)
            or (name in {"skip", "skipIf", "skipUnless"} and len(d.args) >= 1 and name == "skip")
            or (name in {"skipIf", "skipUnless", "skipif", "xfail"} and len(d.args) >= 2)
        )
        out.append((name, has_reason))
    return out


def walk_body(fn):
    """Walk a test function's body without entering nested defs/classes."""
    skip = (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Lambda)
    stack = [s for s in fn.body if not isinstance(s, skip)]
    while stack:
        n = stack.pop()
        yield n
        for c in ast.iter_child_nodes(n):
            if isinstance(c, skip):
                continue
            stack.append(c)


def scan(root):
    c = dict(files=0, tests=0, loop=0, branch=0, try_=0, zero_assert=0, asserts=0,
             assert_stmt=0, assert_stmt_msg=0, multi_assert=0, sleep=0, mock=0,
             private=0, skip_deco=0, skip_deco_reason=0, skip_call=0, xfail_deco=0,
             subtest=0, parametrize=0, docstring=0, return_value=0, fixture_params=0)
    for dp, dn, fns in os.walk(root):
        dn[:] = [d for d in dn if d not in {".git", "node_modules", "build", "_vendor", "vendored"}]
        for f in fns:
            if not f.endswith(".py") or not (f.startswith("test_") or f.endswith("_test.py")):
                continue
            path = os.path.join(dp, f)
            try:
                tree = ast.parse(open(path, encoding="utf-8", errors="replace").read())
            except (SyntaxError, ValueError):
                continue
            c["files"] += 1
            fns_ = []
            for node in tree.body:
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    fns_.append((node, False))
                elif isinstance(node, ast.ClassDef):
                    for m in node.body:
                        if isinstance(m, (ast.FunctionDef, ast.AsyncFunctionDef)):
                            fns_.append((m, True))
            for fn, in_class in fns_:
                if not (fn.name.startswith("test") if in_class else fn.name.startswith("test_")):
                    continue
                c["tests"] += 1
                for name, reason in deco_names(fn):
                    if name in {"skip", "skipif", "skipIf", "skipUnless"}:
                        c["skip_deco"] += 1
                        c["skip_deco_reason"] += reason
                    if name in {"xfail", "expectedFailure"}:
                        c["xfail_deco"] += 1
                    if name == "parametrize":
                        c["parametrize"] += 1
                if ast.get_docstring(fn):
                    c["docstring"] += 1
                params = [a.arg for a in fn.args.args if a.arg not in {"self", "cls"}]
                c["fixture_params"] += bool(params)
                loop = branch = try_ = sleep = mock = private = subtest = ret = False
                n_assert = 0
                for n in walk_body(fn):
                    if isinstance(n, (ast.For, ast.AsyncFor, ast.While)):
                        loop = True
                    elif isinstance(n, (ast.If, ast.IfExp, ast.Match)):
                        branch = True
                    elif isinstance(n, ast.Try):
                        try_ = True
                    elif isinstance(n, ast.Assert):
                        n_assert += 1
                        c["assert_stmt"] += 1
                        c["assert_stmt_msg"] += n.msg is not None
                    elif isinstance(n, ast.Call):
                        name = callee_name(n)
                        if is_assert_call(n):
                            n_assert += 1
                        if name == "sleep":
                            sleep = True
                        if name in MOCK_NAMES:
                            mock = True
                        if name in {"subTest", "test"} and isinstance(n.func, ast.Attribute) and (
                            name == "subTest" or getattr(n.func.value, "id", "") == "subtests"):
                            subtest = True
                        if name in {"skip", "skipTest"}:
                            c["skip_call"] += 1
                    elif isinstance(n, ast.Attribute):
                        a = n.attr
                        base = getattr(n.value, "id", "")
                        if a.startswith("_") and not a.startswith("__") and base not in {"self", "cls", "sys", "os"}:
                            private = True
                        if base == "mock" or a in {"patch", "MagicMock"}:
                            mock = True
                    elif isinstance(n, ast.Name) and n.id in {"monkeypatch", "mocker"}:
                        mock = True
                    elif isinstance(n, ast.Return) and n.value is not None and not (
                        isinstance(n.value, ast.Constant) and n.value.value is None):
                        ret = True
                if "monkeypatch" in params or "mocker" in params:
                    mock = True
                c["loop"] += loop
                c["branch"] += branch
                c["try_"] += try_
                c["sleep"] += sleep
                c["mock"] += mock
                c["private"] += private
                c["subtest"] += subtest
                c["return_value"] += ret
                c["asserts"] += n_assert
                c["zero_assert"] += n_assert == 0
                c["multi_assert"] += n_assert > 1
    return c


if __name__ == "__main__":
    r = scan(sys.argv[1])
    t = r["tests"] or 1
    pct = lambda k: f"{r[k]} ({100 * r[k] / t:.1f}%)"
    print(f"files={r['files']} tests={r['tests']}")
    for k in ["loop", "branch", "try_", "zero_assert", "multi_assert", "sleep", "mock", "private",
              "subtest", "docstring", "return_value", "fixture_params"]:
        print(f"{k}: {pct(k)}")
    print(f"assert_stmt={r['assert_stmt']} with_msg={r['assert_stmt_msg']} "
          f"({100 * r['assert_stmt_msg'] / (r['assert_stmt'] or 1):.1f}%)")
    print(f"skip_deco={r['skip_deco']} with_reason={r['skip_deco_reason']} xfail_deco={r['xfail_deco']} "
          f"skip_call={r['skip_call']} parametrize_deco={r['parametrize']}")
```

</details>
