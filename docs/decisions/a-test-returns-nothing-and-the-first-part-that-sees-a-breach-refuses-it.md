# A test returns nothing, and the first part that sees a breach of a test refuses it

## Context and Problem Statement

[`CONTEXT.md`](../../CONTEXT.md) defines no test, so no part can refuse a function that is not one.
Selection takes each `def test…`, and execution calls it.

| fact | source |
|---|---|
| Bill Wake named the steps "Arrange: Set up the object to be tested", "Act: Act on the object (through some mutator)" and "Assert: Make claims about the object" | [3A – Arrange, Act, Assert](https://xp123.com/3a-arrange-act-assert/) |
| pytest calls the act "the singular, state-changing action that kicks off the behavior we want to test", and adds a cleanup, "where the test picks up after itself" | [pytest's anatomy of a test](https://docs.pytest.org/en/stable/explanation/anatomy.html) |
| a test that runs an assert and then returns `False` prints `passed`, and an `async` test and a generator test print `unknown: no assert ran` | [the research of test interfaces, as checked again](https://github.com/kalonji-tools/purba/issues/688#issuecomment-6452745989), on the wheel from `main` at `d369fee` |
| `def test_c(x)` prints `raised: TypeError: …` | [the table of shapes](https://github.com/kalonji-tools/purba/issues/670), on the same wheel |
| `unittest`, Ward and Vitest pass a test whose body never ran | [the research of test interfaces](https://github.com/kalonji-tools/purba/blob/d69fba8fd2c6a8705829aecd9af2a19ad52695ba/research/test-interface.md) |
| a test that replaces `warnings.showwarning` through a helper passes and prints no warning, and the same test without the helper prints `UserWarning: loud` | [probe 2](https://github.com/kalonji-tools/purba/issues/1082#issuecomment-6962528365), whose `warn2.sh` runs both on the wheel from `main` at `b2e363ab` |
| a test that calls `logging.captureWarnings(True)` passes, and prints nothing on stderr | [the probe of what execution leaves unwatched](https://github.com/kalonji-tools/purba/issues/1082#issuecomment-6962099556), whose `run.sh` runs it on the same wheel |
| under `PYTHONWARNINGS=error`, a test that replaces `warnings.showwarning` gives `raised: UserWarning: loud`; with no such variable, a replacement that raises gives `raised: RuntimeError: shown` | [probe 2](https://github.com/kalonji-tools/purba/issues/1082#issuecomment-6962528365) |
| a test that registers a function with `os.register_at_fork(after_in_child=…)` gets `[readings-differ]` | [the probe of what execution leaves unwatched](https://github.com/kalonji-tools/purba/issues/1082#issuecomment-6962099556) |
| purba's fork runs the functions that a test registers with `os.register_at_fork`, as `os.fork` does | [`fork.rs`](https://github.com/kalonji-tools/purba/blob/b2e363ab/crates/execution/src/fork.rs?plain=1#L20-L43) |
| on one machine and CPython 3.12.14, in ten rounds, 40 cases that only assign take a median of 153.5 ms watched and 71 ms unwatched, and 40 cases whose act runs `difflib` take a median of 352 ms watched and 290.5 ms unwatched | [probe 2](https://github.com/kalonji-tools/purba/issues/1082#issuecomment-6962528365), whose output shows each round |
| on the same machine, in five rounds, 10 cases whose act makes 20,000 calls to a function of the test module take a median of 1,396 ms watched and 567 ms unwatched | [probe 3](https://github.com/kalonji-tools/purba/issues/1082#issuecomment-6963072388), whose output shows each round |
| the documentation of `sys.monitoring` names a tool for the ids 0, 1, 2 and 5, and none for 3 and 4 | [CPython's `sys.monitoring.rst`](https://github.com/python/cpython/blob/v3.12.14/Doc/library/sys.monitoring.rst?plain=1#L70-L80) |
| on the wheel of `main` at `75df252c`, on CPython 3.12.14 and 3.14.4, a module that takes the tool ids 3 and 4 as it loads, a test that sets the events of the tool id 3 on a function to 0, and a module whose test calls a function one way, under a `sitecustomize` that takes 3 and 4, each print `passed`; that module alone gets `[second-path]` | [the probe of the tool ids](https://github.com/kalonji-tools/purba/issues/1084#issuecomment-6963636690) |
| on one machine and CPython 3.12.14, in five rounds, 10 cases whose act makes 20,000 calls to a function of the test module take a median of 377 ms on `main` and 631 ms on a build that reads the events of each function after each statement; with 200 more functions in the module, 396 ms and 3,598 ms | [probe 2 of the tool ids](https://github.com/kalonji-tools/purba/issues/1084#issuecomment-6964071736) |
| an assignment replaces `sys.monitoring.set_local_events`, on CPython 3.12.14 and 3.14.4 | [probe 2 of the tool ids](https://github.com/kalonji-tools/purba/issues/1084#issuecomment-6964071736) |
| on the same machine, the module of 10 cases takes a median of 387 ms on `main` and 391 ms with the wrapper; the module with 200 more functions takes 396 ms and 395 ms | [probe 3 of the tool ids](https://github.com/kalonji-tools/purba/issues/1084#issuecomment-6964246485) |
| a test that binds `sys.monitoring.set_local_events` to a name in a `sitecustomize`, and sets the events of `pick` to 0 through that name, prints `passed` with the wrapper; the same call through `sys.monitoring` gets `[hook-replaced] line 11` | [probe 4 of the tool ids](https://github.com/kalonji-tools/purba/issues/1084#issuecomment-6964498683) |
| on CPython 3.14.4, a test that frees the tool id 3, clears it, or sets its callbacks to `None` gets `[second-path] line 12 did not run` or `line 13 did not run` | [the probe of the tool ids](https://github.com/kalonji-tools/purba/issues/1084#issuecomment-6963636690) |
| execution watches the start of each function of the case, and each call that code of the case makes | [`paths.rs`](https://github.com/kalonji-tools/purba/blob/b2e363ab/crates/execution/src/paths.rs?plain=1#L118-L123) and [`called`](https://github.com/kalonji-tools/purba/blob/b2e363ab/crates/execution/src/paths.rs?plain=1#L245-L258) |

## Considered Options

- **Selection takes each `def test…`, and execution ignores what the call returns.** A test that returns `False` prints `passed`, and an `async` test prints `unknown`.
- **A test takes no input until fixtures has its design in depth.** It contradicts the design of fixtures on the day that the design merges.
- **Two inputs that name one fixture share one object, and purba allows it.** A test that compares two inputs then compares one object with itself.
- **Execution gives `unknown` to a test that returns a value.** Selection sees the `return` first.
- **A test names a shared value whose content can change.** Each test that names it then carries the risk.
- **Each assert at a test's end, except a tuple assert, is an expectation.** An `assert x` checks only that `x` is true. Its line shows no expected value.
- **An expected value may be a name that the module binds once to a literal.** The assert then hides the value.
- **The rules of a test's code judge only a test with no inputs.** An input would then free a test from them.
- **"Act" names everything that a test runs before its checks.** The convention names the arrange and the act apart, and purba keeps its words.
- **Execution watches a helper in another module of the test's folder too.** [Prototype 980](https://github.com/kalonji-tools/purba/blob/9a9fa15/prototypes/980-rules-at-run-time/LEARNINGS.md#held) found the test module alone enough.
- **purba guards the handler of `SIGALRM` too.** No time limit of purba stands on that handler, so a test that replaces it hides no verdict.
- **A `try` of another shape is refused for its shape.** A `try` with a `finally` whose lines all ran shows no fault, and passes.
- **A hook that a line of the test replaces fails the case.** A breach of a test is the test's fault, and `failed` names a fault of the code under test.
- **A `with` that swallows a raise and skips a line fails the case.** The lines that it skips are a path that the case did not take.
- **Execution judges each line of a generator under `contextlib.contextmanager`, and also the lines of its cleanup.** The rule names only the lines before the first expectation. A `with` that holds an expectation runs the cleanup after that expectation.
- **Only a `with` of the test makes a generator under `contextlib.contextmanager` code that the test calls.** Execution would then not judge a generator that a helper enters.
- **An assert inside a reading passes when it holds.** The rule judges where the test asserts, not whether the assert holds.
- **A refused line names the helper's line first, then the test's line, on the same line or on a second, indented line.** The tester reads the test first.
- **A refused line names each line of the test that reaches the helper.** The first line is enough to find the helper.
- **Execution reads the events of each function after each statement before the first expectation.** On a module with 200 more functions, that read added about 320 ms a case, as [probe 2 of the tool ids](https://github.com/kalonji-tools/purba/issues/1084#issuecomment-6964071736) measured.
- **Execution reads the events of each function once a case.** [Probe 2 of the tool ids](https://github.com/kalonji-tools/purba/issues/1084#issuecomment-6964071736) measured no cost of it. A test that sets the events to 0 in one statement and back in a later one would pass. A test that leaves them at 0 would get a refused line that names no line of the test. This is a judgement, from the code that each would run.
- **Execution puts the events back after it refuses a case.** Each case runs in a process of its own, and that process ends with the case. So no later case could see them, as [the test critique of the tool ids](https://github.com/kalonji-tools/purba/pull/1085#issuecomment-6963959117), finding 2, showed.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md` `/LANDSCAPE.md`

**Area:** selection

**A [test](../../CONTEXT.md#test) returns nothing, and the first part that sees a breach of a test refuses it.**
The text of a test shows a `return` of a value before any test runs, so selection refuses it.

| a test | a breach | the part that sees it |
|---|---|---|
| names the fixture of each [input](../../CONTEXT.md#input) in `use(…)` | a parameter with no default | selection |
| | an input that fixtures cannot build | execution |
| | two inputs that name one fixture | execution |
| | an input whose fixture builds a [stock](../../CONTEXT.md#stock), which only a fixture takes | execution |
| returns nothing | a `return` of a value in the test's own body | selection |
| is not a generator | a `yield` in the test's own body | selection |
| can be `async`, and purba cannot run one yet | an `async def` | selection |
| is a function of the module, and not of a class | a `def` with a `test` name in the body of a class, unless the class sits in a `def` | selection |
| states each [expectation](../../CONTEXT.md#expectation) as one comparison, by one operator, with a literal on one side, or as `not x`, `isinstance(x, T)`, `x.startswith(y)` or `x.endswith(y)`, or as an `all()` or an `any()` of one of these | an expectation that is none of these, a chained comparison, or an `all()` or an `any()` with an `if`, a second `for`, a list comprehension or a `:=` | selection |
| | a comparison with no literal on either side, in an `all()` or an `any()` too | selection |
| | an `all()` whose reading holds no item | execution |
| reads in an expectation no iterator that the test's body bound, other than to a parameter | a name that an expectation reads, which holds an iterator in the test's frame | execution |
| takes each [stand-in](../../CONTEXT.md#stand-in) as an argument | a [patch](../../CONTEXT.md#patch) in the test's body, a nested `def`, `lambda` or `class` included | selection |
| | a mock with no spec in the test's body | selection |
| sits in a module that imports no other test framework | an import of pytest in its module outside every test, or in its body, by an import line or by a call that names it in a string | selection |
| expects a raise by more than its type | an `except` in its body that only passes | selection |
| takes one path in the lines before its first expectation, and in each line of the test module that those lines call by name or enter as a generator under `contextlib.contextmanager` | a line that did not run, or a branch that went one way only | execution |
| | a `with` that swallowed a raise and skipped a line | execution |
| asserts only in its expectations | a marked assert that runs before the first expectation, outside the cleanup of a `with` | execution |
| | a marked assert that runs inside a reading, whether it holds or not | execution |
| keeps purba's hooks, `threading.excepthook` and `sys.unraisablehook`, the warning filters that the run began with, and the tool id of `sys.monitoring` that execution watches with and its events | a hook or the filters that a statement replaced before the first expectation, or that the module replaced as it loaded | execution |
| | a module that took, as it loaded, each free one of the tool ids 3 and 4 of `sys.monitoring` | execution |
| | a statement before the first expectation that changed the events of execution's tool id on a function | execution |

- **Selection judges each `def`, under any decorator.** It refuses a test that carries a decorator that is not purba's, unless a suppression names `foreign-decorator`, as [a test carries only purba's own decorators](a-test-carries-only-purbas-own-decorators.md#decision-outcome) says.
- **A `return` in a nested `def`, `lambda` or `class` belongs to it, and not to the test.**
- **Selection refuses a test in a class, whatever the name or the bases of its class.** Its refused line names the test as `path::Class::method`, with each class on the way. The line gives each reason of its `def` and its body, but none for `self`. A binding of its name at the module's level and a statement above its class give no reason either. Execution calls a test by the name of one function.
- **`assert all(<check> for <target> in <reading>)` is an expectation, and so is `assert any(…)`.** Its generator holds one `for`, not `async`, no `if`, no `:=`, and a `<check>` of one comparison or one of the four forms. The expected side of `<check>` keeps the rule of an expected value.
- **An `all()` or an `any()` with an `if`, a second `for`, a list comprehension or a `:=` gets the check `not-one-comparison`.** Its line names the shape: `[not-one-comparison] line 22 is an all() with an if, and an all() takes one for, no if`. For a second or a third `for`, the words say `with two for` or `with three for`. Four or more `for` give `with more than three for`. A list comprehension gives `of a list, not a generator`, and a `:=` gives `with a :=`. Any other `all()` or `any()`, such as one of a name, gets the words that any other expectation gets.
- **An `all()` whose reading holds no item is a project error under the check `no-item`.** Its line reads `[no-item] line 27 checks each item of xs, and xs holds no item`. A passed case would stand on no item that purba checked, so no suppression reaches `no-item`. An `any()` of no item fails, as Python's `any()` does.
- **An expectation that reads a name that holds an iterator is a project error under `iterator-read`, at the first expectation that reads the name.** The test's frame binds the name, and the name is no parameter of the test. The type of its value has `__next__`, and `iter(value) is value` holds. The value is no `io.IOBase`, such as a file or a `StringIO`. The line reads `[iterator-read] line 32 reads it, an iterator that one reading empties; make it a list in the act`. One reading of an iterator gives a true outcome today, so `iterator-read` sits in the practice tier.
- **`iterator-read` does not refuse an iterator that the expectation makes itself by a call.** `assert list(iter(rows)) == [1, 2]` reads it once.
- **The rules of a test's code judge every test, with inputs or without.** An input on either side of a comparison is no literal. Its refused line names the input: `[computed-expected] line 66 compares to the input expected, and an expected value is a literal until purba has cases`.
- **A refused line names each fault of a suppression above the test first.** It then names each reason of the `def`, and each reason of the body by its line number. A test in a class also names each import of pytest outside every test, as [a test module imports no other test framework](a-test-module-imports-no-other-test-framework.md#decision-outcome) says.
- **Selection reports each [private name](../../CONTEXT.md#private-name) in a test's body, and refuses none.** The body takes in a nested `def`, `lambda` or `class`. The case runs. After the lines of its outcome, one [notice](../../CONTEXT.md#notice) shows for each name of each line: `⚠ line 12 reads the private name _default_encoder`. A test that selection refuses for another reason shows its refusal alone.
- **A `try` of one path in a test's body holds one simple statement and one `except SomeError:`, with no `else` and no `finally`.** Its handler may bind the raise with `as`, and names its type by a name or a dotted name. The lines of its `except` that did not run are no fault. A raise of another type is `raised`.
- **When the statement of a `try` of one path does not raise, the case fails.** Its first quote reads `the try at line 38 expects ValueError, and the statement did not raise it`. purba still reads each expectation, and shows each that did not hold after that quote.
- **A name that the `except` binds needs a value before the `try`.** `got = None` before the `try`, and `got = str(e)` in the `except`, give `the try at line 70 expects Closed, and the statement did not raise it` when the statement does not raise.
- **Execution judges any other `try` by its paths, and so a `try` in a helper.** When a line of a `try` of another shape in a test's body did not run, the refused line ends with `; a try of one path holds one statement and one except SomeError:, with no else and no finally`.
- **The words of each breach of a [strict check](../../CONTEXT.md#strict-check) start with the name of the check in brackets, such as `[second-path]`.** [A suppression is an outcome and not a switch](a-suppression-is-an-outcome-and-not-a-switch.md#decision-outcome) holds each name and its tier.
- **A suppression of a check of the [practice tier](../../CONTEXT.md#practice-tier) lets the test run.** The case then gets the outcome that it would get without that check.
- **Selection leaves two inputs that name one fixture to execution.** Execution refuses such a test, so no test gets one object under two names. Only execution resolves the names in `use(…)`.
- **The type of a [catalyst](../../CONTEXT.md#catalyst) proves it immutable.** Fixtures reads the type before it builds the value.
- **A test reaches a stock only through a [reagent](../../CONTEXT.md#reagent) that a fixture draws from it.** The risk that a test changes what the next test sees sits in one fixture. That fixture undoes what the test changes, such as through one transaction that it rolls back.
- **A test follows [arrange, act, assert](../../CONTEXT.md#arrange-act-assert).** An expectation inside a `with` reads before the `with` cleans up, and one after it reads once the cleanup ran.
- **A tuple assert with items carries no mark, so execution does not refuse it before the first expectation.** [A tuple assert keeps CPython's warning, and selection refuses a test that ends with one](a-tuple-assert-keeps-cpythons-warning-and-counts-as-no-assert.md#decision-outcome) says why it carries none.
- **Execution watches only the test module's code, and a function of it only when code of the case calls it by name.** The last call that the caller made names the function, a bound method of it, or a class whose `__init__` or `__new__` it is. Execution does not watch a function that C code calls back, such as the `key` of `sorted`. It also does not watch code that a library calls back, or a module that the test imports. It lets a loop's head, a `with`'s exit, an `except`'s match and an assert's test go one way. A line of a loop's body shows that the loop ran a round, and the mark of an assert shows that the assert ran.
- **A generator under `contextlib.contextmanager` can be code that the test calls.** It is when the test, or code that the test calls, enters it by a `with` or `ExitStack.enter_context`. Execution judges each line of it, except its cleanup. Its cleanup is the lines after the `yield` where it waits at the first expectation, which run after that expectation.
- **A refused line for a helper names the first line of the test that reaches the helper.** Then it names the line of the helper and the name of the helper. It reads `[second-path] line 33 reaches line 16 of make, which went one way only, and the lines before the first expectation take one path`. For an assert, the line of the test is the one that runs the assert.
- **The refused line for an assert in the act names the fix.** It reads `[assert-before-expectation] line 59: an assert ran in the act; one test checks one state; check this state in a test of its own`. For an assert in a helper, it reads `line 53 reaches line 7 of key, which ran an assert in the act; …`.
- **When a fork or a replay reaches an assert, the refused line names the expectation that it read alone.** A fork carries only its verdict in its exit code. A replay carries its verdict in a frame. The line reads `[assert-in-reading] assert check() is True reaches an assert when read alone, and only an expectation asserts`.
- **purba allows a library that replaces a hook and puts it back inside one call.** purba checks each hook after each statement of the test before its first expectation, and at the end of the module's load. It puts back each hook that it finds replaced. Execution refuses a change of the events of its tool id, even when the same statement puts them back.
- **Execution refuses a case or a module that changes the warning filters.** It compares the filters that apply with those it held when the act or the load began. On CPython 3.14 they can sit in a context variable, which `warnings._get_filters` reads. Another filter, or the same filters in another order, refuses the case or the module: `[hook-replaced] line 47 replaced the warning filters, and a test keeps the filters that the run began with`. Execution refuses a filter inside `with warnings.catch_warnings():` too, so a test cannot hide `PYTHONWARNINGS=error`. purba does not put the filters back, since each case runs in a process of its own.
- **Execution refuses a module that takes each free one of the tool ids 3 and 4 of `sys.monitoring`.** Execution watches with one of these two ids. The documentation of `sys.monitoring` names a tool for 0, 1, 2 and 5, and none for 3 and 4. After the module loads, execution takes the first free one of 3 and 4, so a module that takes one of them still runs. Execution refuses a module that takes each free one: `[hook-replaced] the module took the tool ids 3 and 4 of sys.monitoring as it loaded, and a test keeps purba's hooks`. Where a tool held one before the load, the line names the tool id that the module took.
- **Execution refuses the load when other tools already hold both ids.** The module does not run. The line names each tool: `sys.monitoring holds the tool ids 3 and 4 for coverage and profiler, and execution needs one of them to watch`.
- **Execution refuses a case that changes the events of execution's tool id.** Before the module loads, execution puts a wrapper in the place of `sys.monitoring.set_local_events`. The wrapper sets a flag at each call that changes the events of execution's tool id on a function. After each statement before the first expectation, execution reads that flag and clears it. A set flag refuses the case: `[hook-replaced] line 11 replaced the events of sys.monitoring that purba watches with, and a test keeps purba's hooks`. Execution also refuses a case whose statement changes the events and puts them back. A call that sets the events to the value that they hold changes nothing, so execution allows it. purba does not put the events back, since each case runs in a process of its own.
- **A raise at the last line of a `with`'s body skips no line, so execution allows a `with` that swallows it.** A raise that the case lets through gives the outcome `raised`. Execution counts no line that the raise skips as a path that the case did not take.

**Downside:**

- **Selection refuses a `return` of a value in a branch that never runs.** The call would return nothing.
- **purba cannot check what a fixture does with a stock.** A fixture that undoes nothing lets a test change what the next test sees.
- **A value whose type does not prove it immutable is a stock.** click's `Command` is one, though no outcome of click depends on what a test leaves in it.
- **Selection reports a name that a library documents with a leading `_`, such as `os._exit`, too.**
- **Selection does not see a private name in a string, such as `getattr(w, "_flush")`.**
- **Selection refuses a test that compares two computed values, because neither is a literal.** A test that compares a value with what CPython gives without purba writes that value as a literal.
- **A test that forks takes one way of `if pid == 0` in each process.** A tester calls the child's work through a dictionary of functions, such as `{0: child}.get(pid, int)()`.
- **A `while` that polls may run no round.** A tester polls in a module that the test imports.
- **`x = a or b`, a conditional expression, the `if` of a comprehension and a `match` are branches.** Execution refuses the case unless each goes both ways before the first expectation.
- **A function that the interpreter calls for an operator or an attribute takes any path.** `__eq__` and the getter of a property are such functions, since no call names them.
- **A name that the `except` binds, and that no line before the `try` binds, gives `raised: UnboundLocalError` when the statement does not raise.** The outcome `raised` ranks above a `try` that did not raise, as [a failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) orders them.
- **Each branch of a context manager of the test module goes both ways before the first expectation, or execution refuses the case.** This holds for a class, and for a generator that a `with` or `ExitStack.enter_context` enters. Execution does not judge a context manager from `contextlib.contextmanager` that decorates a function, since `contextlib` enters it.
- **A hook that a reading replaces stays unseen.** It hides the exception of each thread that raises after it.
- **purba does not guard `warnings.showwarning`.** A test that replaces it hides the line of each warning. The replacement cannot hide `PYTHONWARNINGS=error`, since the filter raises before `showwarning` runs. A replacement that raises gives the outcome `raised`.
- **A function that a test registers with `os.register_at_fork` runs at each fork that reads an expectation alone.** It can change what the expectation reads. The case then gets `[readings-differ]`. [No suppression lifts `readings-differ`](a-suppression-is-an-outcome-and-not-a-switch.md#decision-outcome).
- **Execution's watch costs more for each call that the act makes from the test module.** By the medians on one machine, it adds about 2 ms to a case whose act only assigns. It adds about 83 ms to a case whose act makes 20,000 such calls.
- **Code that bound `sys.monitoring.set_local_events` to a name before execution wrapped it goes around the wrapper.** A `sitecustomize` runs before purba, so it can bind the function that the wrapper replaces.
- **The wrapper reads the events of a function at each call that names execution's tool id.** The act of each case in the two modules of [probe 3 of the tool ids](https://github.com/kalonji-tools/purba/issues/1084#issuecomment-6964246485) makes 20,000 calls. None of these calls names execution's tool id. On these modules, the wrapper adds no time that the medians can tell from the noise.
- **On CPython 3.14, a test that frees execution's tool id, clears it, or sets its callbacks to `None` gets the wrong cause.** Execution refuses the case, and its line names a line that ran: `[second-path] line 12 did not run`. That the line ran is a judgement: it binds the name that the expectation on the next line reads.
- **`not-one-comparison` names each `all()` and `any()` that selection refuses for its shape, though none of them is a comparison.**
- **`iterator-read` refuses an expectation that reads an iterator and does not empty it, such as `assert it is not None`.**
- **`iterator-read` refuses `assert isinstance(it, Iterator)` too, and its words advise a list.**
- **`iterator-read` reads no parameter of the test, so a parameter that the act binds again to an iterator escapes it.**
- **`iterator-read` does not see `assert all(it > 0 for it in it)`.** The target of the `for` hides the name that its reading reads.
- **Selection refuses an `all()` whose check holds a `:=`, though Python runs it.** The check runs inside purba's function, so the name would not reach the test.
- **A module that binds its own `all` or `any` still gets the rules of Python's.** Selection reads the name, and not what the module binds to it.
- **`iterator-read` does not see an iterator behind an attribute, such as `shelf.rows`, or behind a name of the module.**
- **An iterator that an expectation makes itself runs the code under test inside the reading.** A raise there gives the outcome `raised`. A line under it names the frame of the raise when that frame is not the test's own.

## Confirmation

`mise run test:rust` checks that selection refuses an `async def`, a `yield`, a `return` of a value and a parameter with no default.
It checks that selection refuses an expectation that is not one comparison or one of the four forms, and one with no literal.
It checks that a private name gives a notice.
It checks that the refused line names each by its line number.
It checks that selection refuses a test in a class, and that its line names each class on the way.

`mise run test:rust` checks in `crates/selection/tests/select.rs` that selection takes an `all()` or an `any()` of each shape of a check. It checks the words that selection prints for each shape that it refuses. It checks in `crates/execution/tests/run.rs` that an iterator that a parameter holds is no fault.

`mise run test:wheel` checks in `tests/rules.bats` that execution refuses a case for each rule that it sees, except `no-item` and `iterator-read`.
It checks that the refused line of a case names the rule and the line of the test.
It checks the refused load of a module that takes the tool ids 3 and 4, and of a module under tools that hold both.
It checks that execution watches a module that takes the tool id 3 alone.

`tests/strict.bats` checks a private name, the four forms and a `try` of one path on one module. It also checks a callback, the warning filters, an assert in the act and an input on the expected side.
For a helper, it checks that the line also names the line of the helper and the name of the helper.
`tests/iterable.bats` checks a failed `all()`, a failed `any()`, an `all()` with an `if`, an `all()` of no item, and an expectation that reads an iterator.

The designs in depth of fixtures and execution, which [the landscape](../../LANDSCAPE.md#the-order-of-design) ranks, build the rest.
[A test names the fixture of each input](a-test-names-the-fixture-of-each-input.md#decision-outcome) says how a test names its inputs.
[What keeps a test from changing an input that other tests share outside its process?](https://github.com/kalonji-tools/purba/issues/692) decides which types prove a value immutable.

[A stand-in enters a test as an argument, so selection refuses a patch and a mock with no spec](a-stand-in-enters-a-test-as-an-argument.md#decision-outcome) says which patches selection refuses.
[A test that expects a raise names more than its type, so selection refuses an except that only passes](a-test-that-expects-a-raise-names-more-than-its-type.md#decision-outcome) says which `except` selection refuses.
[A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) names the kind of each breach.
[An error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome) says what each breach stops.
[Each test runs in a process of its own](each-test-runs-in-a-process-of-its-own.md#decision-outcome) says what may change around a test.
