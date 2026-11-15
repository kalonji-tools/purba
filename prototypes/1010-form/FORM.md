# The rules of this form

A test form, run by its own small runner: `python run.py [file or folder …]`, with the code under test on `PYTHONPATH`.
A tester imports `case`, `raises`, `warns`, `handling` and `use` from `cases.py`, and the bases of a stock (`Samples`, `Checkpoints`, `Fingerprints`), and only what a test needs.
The code under test sits outside the tests' folder.

## A test

- **A test is a plain function named `test…` in a file named `test_….py`. Its parameters are its inputs, and its checks are the asserts at its end.**
  The act is everything before the first check. **The act ends at the first check**, and only checks follow it.

  ```python
  def test_lend(shop: Shop = use(Shop)) -> None:
      order = shop.checkout(shop.sign_up("ann"))   # the act
      assert order.total == 0                       # a check
  ```

- A test whose parameters all have a default, or that has none, runs once under its own name.
- A `return` may follow the checks. It hands a product to a later test: the test is then a **stage** (below).
- **A test may hold as many calls of the code under test as it needs.** No rule counts the acts.

## The checks

- **A check is one comparison: `assert reading == expected`, or with a message, `assert reading == expected, "what it checks"`.**
  Any one operator: `==`, `!=`, `<`, `<=`, `>`, `>=`, `in`, `not in`, `is`, `is not`.
- **One side is the expected value:** a literal, operators on literals (`"7" * 20 + "\n"`), an input of the test, or a list, tuple, set or dict of these.
  The other side reads what the act made, and may compute: `assert writer.read().splitlines()[0] == "a"`.
  A computed expected value is refused: `assert total == price * 2` is refused; write `assert total == 2400`, or take `total` as an input.
- **A reading reads; it does not act.** Each check reads in place, in order, and each check after the first is also read alone. A check that holds alone and fails after the checks before it, or the other way round, is refused.
  Call the code in the act, `wrapped = should_wrap(stream)`, and check `wrapped`.

  ```python
  stream = io.StringIO("x")
  assert stream.read() == "x"
  assert stream.read() == "x"      # refused: the reading before it emptied the stream
  ```

- **An assert that runs inside a reading is refused.** A helper that a check calls returns what it reads, and the check compares it.

  ```python
  assert check_paid(order) is None      # refused: check_paid holds an assert
  assert state_of(order) == "paid"      # good: state_of returns order.state
  ```

- `==` compares by type, then by value: the reading must be an instance of the expected value's type, `True` is never `1`, and `1.0` is never `1`. Lists, tuples and dicts are compared item by item.
- Every check runs, even after another fails. A failure names the check by its own text, and prints its message if it has one.
- A check that reads only an input holding a `str`, a number, a `bool`, `bytes`, `None` or a tuple is refused: the act cannot change it.

## The act has one path

- **Each line of the act runs in every case. Each branch in it (`if`, `x if c else y`, `and`, `or`, `while`, `except`) goes both ways in a case, or is not there. Each loop runs at least one round.**
  A case that breaks this is refused, and the runner names the line. A loop over any data is fine: `for t in threads: t.join()`.

  ```python
  def test_name(name: str = "") -> None:
      shown = name or "anonymous"      # refused: the `or` goes one way
  ```

- **The runner watches the code of the test module only:** the test, and each function or class of the same file that the test calls, at any depth. It also watches a `@contextmanager` function of the same file that a `with` of the test enters.
  Code of any other file is not watched: the code under test, and a helper module, wherever it sits.
- A function of the test module that **the code under test calls** (a callback, a sink, a format function) is a value: its paths are free. Who calls it decides, not how it was handed over.
- **An `assert` that runs in the act is refused,** in a helper or a callback too: the act ends at the first check, and the code under test may swallow an assert. Keep what the callback sees, and check that at the end.

  ```python
  def sink(message: str) -> None:
      seen.append(message)             # good
  def sink(message: str) -> None:
      assert message                   # refused
  ```

- **A refusal in a helper names the helper's line and the test's line that reached it:** `the branch at test_shop.py:8 (pick, reached from line 21 of the test) went one way`.
- A `raise` may sit in the act. A setup that swallows it at its last line passes; any other raise ends the act.

## A `with` in the act

- **A `with` in the act is a setup:** what it sets up holds for the statements inside it, and is undone when they end.

  ```python
  with freeze_time("2001-02-03"):
      stamp = log.first_line()
  assert stamp == "2001-02-03"
  ```

  - The checks may sit at the end of a setup, after it, or both. Checks at its end read under it; checks after it read once it is undone.
  - A setup may hold several items, or another `with`. A `with` inside a loop or an `if` is refused.
  - A setup that swallows a raise (such as `contextlib.suppress`) fails the case when a statement inside it never ran because of it. A raise in its last statement may be swallowed.
  - A setup whose own entry or undo raises fails the case, and names it.
- **Three `with` blocks of the form each wrap one statement, and each takes a `with` of its own:**
  - `with raises(SomeError, match=r"…") as caught:` expects a raise. It ends the act: a line of the act that runs after it caught is refused. The checks may read `caught.value`, the exception. `as caught` is optional.

    ```python
    with raises(NotOnShelf, match="Dune") as caught:
        library.lend(member, "Dune")
    assert str(caught.value) == "'Dune' is not on the shelf"
    ```

  - `with warns(SomeWarning, match=r"…"):` expects a warning.
  - `with handling(SomeError, call, *args):` is for code that must run inside an `except`, such as `logger.exception`. It fails the case unless the statement raises `SomeError`, and then runs `call(*args)` while that exception is live.

## Cases

- **A case is `name = case(test, input=…, …)` at module level, named by its variable.** Each input is named; a parameter with a default may be left out.

  ```python
  ann = case(test_join, name="ann")
  bob = case(test_join, name="bob")
  ```

- A type checker reads the call against the test's parameters. The runner checks each input's type too, when it can read the annotation at run time.
- An input that a check uses as its expected value takes a literal in every case.
- To pass a value of the wrong type on purpose, declare the input `object`.
- A case that no module name holds (bound again later, built by a loop, never bound), and a case that two names hold, is refused.

## Fixtures

- **A fixture is the default of an input, `use(factory)`.** The runner calls `factory()` again for each case. A case may pass its own value instead.

  ```python
  def stocked() -> Library:
      library = Library()
      library.stock("Dune", "Emma")
      return library

  def test_join(name: str, library: Library = use(stocked)) -> Member: ...
  ```

- A fixture may call the code under test. No check sees what it did: when a check should, make it a stage.
- **A reset of global state is a fixture too,** named in the signature of each test that needs it: `_reset: None = use(reset_logger)`. Nothing runs after a case: its process ends, and what it changed ends with it.

## Stages

- **A stage is a case whose test checks its own act and returns a product. A later test takes that product as the default of an input, `use(stage)`:**

  ```python
  def test_join(name: str, library: Library = use(stocked)) -> Member:
      member = library.join(name)
      assert library.members == [name]
      return member

  ann = case(test_join, name="ann")

  def test_borrow(member: Member = use(ann)) -> Loan:
      loan = member.library.lend(member, "Dune")
      assert loan.due == 21
      return loan
  ```

- A case line may also give a product: `case(test_borrow, member=ann.product)`.
- A stage on a stage makes a pipeline. Each stage checks its own step, and a test never repeats another test's checks.
- **A stage may come from another test module:** `from test_members import ann`. The runner loads that module as it loads a test module, and runs the stage with every rule.
- **Each stage's act runs once.** The runner forks the stage's process for its checks, and once for each later stage, which takes the product from that process.
  It runs the stage again for each later stage instead (a replay) when, after the act, a thread other than the main one is alive, the chain reads a stock, or the act left a file, socket or pipe open. The report marks such a later stage with `↺` and says why.
- **A stage that failed, that the runner refused, or that was blocked blocks each later stage.** The line names it: `⊘ test_on_time  blocked: it takes the product of two_weeks (test_due_dates.py:17), which failed`.
- A stage whose test returns nothing is refused before the run.

## Stocks

- **A stock is an input whose state lives outside the process,** such as a database file. Its fixture subclasses `Samples[T]` (`sample()` for each case, `discard(value)` after it), `Checkpoints[T, M]` (`open()`, `checkpoint(value)`, `restore(value, mark)`), or both. A test takes it as `db: Path = use(UsersDb)`.
- For each check, the fixture also subclasses `Fingerprints[T]` (`fingerprint(value)`) or `Checkpoints`. A stock that offers neither is refused before the run. A check whose reading changed a stock is refused, through whatever name it read it.

## Each case runs alone

- **Each case runs in a forked process of its own,** so nothing a case does reaches another. Output a case prints is shown only when it fails.
- An open file, socket, pipe or `mmap` that two checks read is refused: a fork cannot seal it.
- A warning that the test does not expect fails the case, and so does an error that no code sees (in a thread, or in a finaliser). To expect a warning, use `warns`.
- **The act may not replace a hook that the runner sees a fault through:** `threading.excepthook`, `sys.unraisablehook`, the warning filters (`warnings.simplefilter`, `catch_warnings`, …) and the time limit (`signal.alarm`, a `SIGALRM` handler). `sys.stdout` and `sys.stderr` are free.
- The runner imports a test file with each check rewritten as a check of its own. Called by hand, a test is a plain function, and its asserts are Python's.

## What the runner refuses, and when

- **Before the run, from the code:** a check in another form, a computed expected value, a check that reads only a value the act cannot change, a `global`, `nonlocal`, `yield`, `await` or `async with`, a `with` inside a loop or another block, a test with a decorator, and a stock with no watch. A refusal names every fault at once.
- **After the import:** a misspelled or missing input (with a hint), an input of the wrong type, an input by position, a case with no name or with two, and a stage that returns nothing.
- **While the case runs:** a path of the act that a case did not take, an assert in the act or in a reading, a line after an expected raise, a check whose reading changed what a later check reads or changed a stock, and a case in which no check ran.
- A refusal is the test's fault; a failure is the code's. A case with a refusal counts as refused, even if a check also failed.

## The report

- `✓` a check held · `✗` a check failed · `!` refused · `⊘` blocked · `↺` ran by replay.
- The run ends with a count, `N cases: … passed · … failed · … refused · … blocked`, and exits 1 unless every case passed.
