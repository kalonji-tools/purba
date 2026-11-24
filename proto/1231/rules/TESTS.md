# How purba finds and runs a test

purba is a test runner for Python. It reads each test file as text first, and imports only the files whose tests it will run.

## A test file

A file whose name is `test_*.py` or `*_test.py`, anywhere under the folder where purba starts. A folder whose name starts with `.`, and `__pycache__`, are skipped.

## A test

A function at the top level of a test file whose name starts with `test`, such as `test_format` or `testformat`. purba calls it with no arguments.

- A function with a parameter that has no default is not called. purba reports it as a project error: "the parameter writer has no default, and purba cannot see which fixture builds it". purba has no fixtures and no parametrize yet, so build each value inside the test, or in a helper that the test calls.
- A method of a class is not called. purba reports it as a project error.
- A decorator other than purba's own (see `TAG.md`) is refused: "[foreign-decorator] the decorator … is not purba's, and purba cannot see what it does". So is a test that uses another test framework.
- Each test is one case, under its own name. A loop over several inputs inside one test is still one case.
- A function whose name does not start with `test` is a helper. purba never calls it as a test.

## An expectation

purba gives a case its outcome from the asserts at the end of the test's body. Those asserts are its expectations.

- The expectations come last. Only other expectations may follow one. An assert followed by more code, also one inside a loop, blocks the case as it runs: "[assert-before-expectation] … an assert ran in the act; one test checks one state; check this state in a test of its own".
- An expectation may also close the body of a `with` that comes just before the last asserts.
- An expectation compares two values by one operator, with a literal on one side: `assert writer.read() == "a\n"`, `assert len(lines) > 1`.
- Or it takes one of four forms: `assert not x`, `assert isinstance(x, T)`, `assert x.startswith(y)`, `assert x.endswith(y)`.
- Or it checks each item of an `all()` or an `any()` by one of these.
- Anything else, such as `assert ymd.has_month`, is refused: "[not-one-comparison]". Write `assert ymd.has_month is True`.
- A test in which no expectation runs is blocked, not passed: "[no-expectation]".

## A run

```sh
purba <query>
```

The query names the tests to run; `QUERY.md` holds its rules. `purba tests` runs every test under `tests/`.

Each case prints one line: `passed`, `failed`, `raised: <exception>`, or `project error: <why>`. A failed case prints the assert, the expected value and the reading. The last line counts the cases.

| exit code | meaning |
|---|---|
| 0 | every case passed |
| 1 | a case failed or raised |
| 3 | the command was wrong, such as a query that picks no test |
| 4 | a project error, such as a refused test, and nothing failed |

A line on stderr that starts with `purba:` is a message from purba itself. `NAME-NEAR.md` and `RERUN.md` describe two of them.
