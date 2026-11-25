# How purba finds and runs a test

purba is a test runner for Python. It reads each test file as text first, then runs each test in a process of its own.

## The root

purba runs from the root of a project. `ROOT.md` says how purba finds it. Each test's id starts at the root, such as `tests/test_a.py::test_add`.

## A test file

A file whose name matches `test_*.py`, anywhere under the root. `TEST-FILES.md` says how to choose other names. purba skips a folder whose name starts with `.`, and each file or folder that a `.gitignore` of the project ignores.

## A test

A function at the top level of a test file whose name starts with `test`, such as `test_format` or `testformat`. purba calls it with no arguments.

- A function with a parameter that has no default is not called. purba reports it as a project error: "the parameter writer has no default, and purba cannot see which fixture builds it". purba has no fixtures and no parametrize, so build each value inside the test, or in a helper that the test calls.
- A method of a class is not called: "purba cannot call a method of a class".
- A decorator is refused: "[foreign-decorator] the decorator functools.cache is not purba's, and purba cannot see what it does".
- Each test is one case, under its own name.
- A function whose name does not start with `test` is a helper. purba never calls it as a test.

## An expectation

purba gives a case its outcome from the asserts at the end of the test's body. Those asserts are its expectations.

- The expectations come last. Only other expectations may follow one. An assert followed by more code, also one inside a loop, is refused: "[assert-before-expectation] line 24: an assert ran in the act; one test checks one state; check this state in a test of its own".
- An expectation may also close the body of a `with` that comes just before the last asserts.
- An expectation compares two values by one operator: `assert writer.read() == "a\n"`, `assert len(lines) > 1`.
- Or it takes one of four forms: `assert not x`, `assert isinstance(x, T)`, `assert x.startswith(y)`, `assert x.endswith(y)`.
- Or it checks each item of an `all()` or an `any()` by one of these.
- Anything else is refused: "[not-one-comparison] line 37 is not one comparison, and an expectation compares a reading by one operator, or reads not x, isinstance(x, T), x.startswith(y) or x.endswith(y)". Write `assert ready is True`, not `assert ready`.
- The lines before the first expectation take one path. An `if` there is refused: "[second-path] line 8 went one way only, and the lines before the first expectation take one path".
- A test with no expectation is refused: "[no-expectation] no expectation stated".

## A run

```sh
purba [<path>]
```

With no path, purba runs every test file under the root, from any folder under it. A path names one test file or a folder, and starts at the root, not at the working folder: from `tests/`, write `purba tests/deep`, not `purba deep`. purba takes no option: `purba --help` is read as a path, which names no test file. Two paths are refused: "usage: purba [<path>]".

Each case prints one line: `passed`, `failed`, `raised: <exception>`, or `project error: <why>`. A failed case prints the assert, the expected value and the reading. The last line counts the cases. It calls a case with a project error "blocked", since it has no outcome.

A test that reads a private name gets a line under its outcome: `⚠ line 5 reads the private name _CONSTANTS`. It changes no count and no exit code.

| exit code | meaning |
|---|---|
| 0 | at least one case ran, and every case passed |
| 1 | a case failed |
| 2 | no case ran |
| 3 | the command was wrong, such as a path that names no test file |
| 4 | a case raised, or purba found a project error, and no case failed |

A line on stderr that starts with `purba:` is a message from purba itself, such as `purba: invocation error: deep names no test file: no file or folder has that path`.
