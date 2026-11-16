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

## Considered Options

- **Selection takes each `def test…`, and execution ignores what the call returns.** A test that returns `False` prints `passed`, and an `async` test prints `unknown`.
- **A test takes no input until fixtures has its design in depth.** It contradicts the design of fixtures on the day that the design merges.
- **Two inputs that name one fixture share one object, and purba allows it.** A test that compares two inputs then compares one object with itself.
- **Execution gives `unknown` to a test that returns a value.** Selection sees the `return` first.
- **A test names a shared value whose content can change.** Each test that names it then carries the risk.
- **Each assert at a test's end, except a tuple assert, is an expectation.** An `assert x` checks only that `x` is true. Its line shows no expected value.
- **An expected value may be a name that the module binds once to a literal.** The assert then hides the value.
- **Selection refuses a private name unless a [suppression](../../CONTEXT.md#suppression) gives a reason.** purba has no syntax for a suppression yet.
- **The rules of a test's code judge only a test with no inputs.** An input would then free a test from them.
- **"Act" names everything that a test runs before its checks.** The convention names the arrange and the act apart, and purba keeps its words.
- **Execution watches a helper in another module of the test's folder too.** [Prototype 980](https://github.com/kalonji-tools/purba/blob/9a9fa15/prototypes/980-rules-at-run-time/LEARNINGS.md#held) found the test module alone enough.
- **purba guards `warnings.showwarning` and the handler of `SIGALRM` too.** purba guards only `threading.excepthook` and `sys.unraisablehook`.
- **A hook that a line of the test replaces fails the case.** A breach of a test is the test's fault, and `failed` names a fault of the code under test.
- **A `with` that swallows a raise and skips a line fails the case.** The lines that it skips are a path that the case did not take.
- **Execution judges each line of a generator under `contextlib.contextmanager`, and also the lines of its cleanup.** The rule names only the lines before the first expectation. A `with` that holds an expectation runs the cleanup after that expectation.
- **Only a `with` of the test makes a generator under `contextlib.contextmanager` code that the test calls.** Execution would then not judge a generator that a helper enters.
- **An assert inside a reading passes when it holds.** The rule judges where the test asserts, not whether the assert holds.
- **A refused line names the helper's line first, then the test's line, on the same line or on a second, indented line.** The tester reads the test first.
- **A refused line names each line of the test that reaches the helper.** The first line is enough to find the helper.

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
| states each [expectation](../../CONTEXT.md#expectation) as one comparison, by one operator, with a literal on one side | an expectation that is no comparison, or a chained comparison | selection |
| | an expectation with no literal on either side | selection |
| writes no [private name](../../CONTEXT.md#private-name) in its body | a private name in the test's body, a nested `def`, `lambda` or `class` included | selection |
| takes one path in the lines before its first expectation, and in each line of the test module that those lines call or enter as a generator under `contextlib.contextmanager` | a line that did not run, or a branch that went one way only | execution |
| | a `with` that swallowed a raise and skipped a line | execution |
| asserts only in its expectations | a marked assert that runs before the first expectation, outside the cleanup of a `with` | execution |
| | a marked assert that runs inside a reading, whether it holds or not | execution |
| keeps purba's hooks, `threading.excepthook` and `sys.unraisablehook` | a hook that a statement replaced before the first expectation, or that the module replaced as it loaded | execution |

- **Selection judges each `def`, under any decorator.** It refuses a test that carries a decorator that is not purba's, as [a test carries only purba's own decorators](a-test-carries-only-purbas-own-decorators.md#decision-outcome) says.
- **A `return` in a nested `def`, `lambda` or `class` belongs to it, and not to the test.**
- **Selection refuses a test in a class, whatever the name or the bases of its class.** Its refused line names the test as `path::Class::method`, with each class on the way. The line gives each reason of its `def` and its body, but none for `self`. A binding of its name at the module's level and a statement above its class give no reason either. Execution calls a test by the name of one function.
- **The rules of a test's code judge every test, with inputs or without.** An input on either side of the comparison is no literal.
- **A refused line names each reason of the `def` first, then each reason of the body by its line number.** Selection names each private name once for each line.
- **Selection leaves two inputs that name one fixture to execution.** Execution refuses such a test, so no test gets one object under two names. Only execution resolves the names in `use(…)`.
- **The type of a [catalyst](../../CONTEXT.md#catalyst) proves it immutable.** Fixtures reads the type before it builds the value.
- **A test reaches a stock only through a [reagent](../../CONTEXT.md#reagent) that a fixture draws from it.** The risk that a test changes what the next test sees sits in one fixture. That fixture undoes what the test changes, such as through one transaction that it rolls back.
- **A test follows [arrange, act, assert](../../CONTEXT.md#arrange-act-assert).** An expectation inside a `with` reads before the `with` cleans up, and one after it reads once the cleanup ran.
- **A tuple assert with items carries no mark, so execution does not refuse it before the first expectation.** [A tuple assert keeps CPython's warning, and selection refuses a test that ends with one](a-tuple-assert-keeps-cpythons-warning-and-counts-as-no-assert.md#decision-outcome) says why it carries none.
- **Execution watches only the test module's code.** It does not watch other code that a library calls back, or a module that the test imports. It lets a loop's head, a `with`'s exit, an `except`'s match and an assert's test go one way. A line of a loop's body shows that the loop ran a round, and the mark of an assert shows that the assert ran.
- **A generator under `contextlib.contextmanager` can be code that the test calls.** It is when the test, or code that the test calls, enters it by a `with` or `ExitStack.enter_context`. Execution judges each line of it, except its cleanup. Its cleanup is the lines after the `yield` where it waits at the first expectation, which run after that expectation.
- **A refused line for a helper names the first line of the test that reaches the helper.** Then it names the line of the helper and the name of the helper. It reads `line 33 reaches line 16 of make, which went one way only, and the lines before the first expectation take one path`. For an assert, the line of the test is the one that runs the assert.
- **When a fork or a replay reaches an assert, the refused line names the expectation that it read alone.** The exit code of the fork or the replay carries only its verdict. The line reads `assert check() is True reaches an assert when read alone, and only an expectation asserts`.
- **purba allows a library that replaces a hook and puts it back inside one call.** purba checks each hook after each statement of the test before its first expectation, and at the end of the module's load. It puts back each hook that it finds replaced.
- **A raise at the last line of a `with`'s body skips no line, so execution allows a `with` that swallows it.** A raise that the case lets through gives the outcome `raised`. Execution counts no line that the raise skips as a path that the case did not take.

**Downside:**

- **Selection refuses a `return` of a value in a branch that never runs.** The call would return nothing.
- **purba cannot check what a fixture does with a stock.** A fixture that undoes nothing lets a test change what the next test sees.
- **A value whose type does not prove it immutable is a stock.** click's `Command` is one, though no outcome of click depends on what a test leaves in it.
- **Selection refuses a name that a library documents with a leading `_`, such as `os._exit`, too.** A tester calls it through a function of the test module, which selection does not read.
- **Selection does not see a private name in a string, such as `getattr(w, "_flush")`.**
- **Selection refuses a test that compares two computed values, because neither is a literal.** A test that compares a value with what CPython gives without purba writes that value as a literal.
- **A test that forks takes one way of `if pid == 0` in each process.** A tester calls the child's work through a dictionary of functions, such as `{0: child}.get(pid, int)()`.
- **A `while` that polls may run no round.** A tester polls in a module that the test imports.
- **`x = a or b`, a conditional expression, the `if` of a comprehension and a `match` are branches.** Execution refuses the case unless each goes both ways before the first expectation.
- **A callback that C code calls counts as code that the test calls.** The `key` of `sorted` is one, since C leaves no Python frame between the test and the callback.
- **Each branch of a context manager of the test module goes both ways before the first expectation, or execution refuses the case.** This holds for a class, and for a generator that a `with` or `ExitStack.enter_context` enters. Execution does not judge a context manager from `contextlib.contextmanager` that decorates a function, since `contextlib` enters it.
- **A hook that a reading replaces stays unseen.** It hides the exception of each thread that raises after it.

## Confirmation

`mise run test:rust` checks that selection refuses an `async def`, a `yield`, a `return` of a value and a parameter with no default.
It checks that selection refuses an expectation that is not one comparison, one with no literal, and a private name.
It checks that the refused line names each by its line number.
It checks that selection refuses a test in a class, and that its line names each class on the way.

`mise run test:wheel` checks in `tests/rules.bats` that execution refuses a case for each rule that it sees.
It checks that the refused line names the rule and the line of the test.
For a helper, it checks that the line also names the line of the helper and the name of the helper.

The designs in depth of fixtures and execution, which [the landscape](../../LANDSCAPE.md#the-order-of-design) ranks, build the rest.
[A test names the fixture of each input](a-test-names-the-fixture-of-each-input.md#decision-outcome) says how a test names its inputs.
[What keeps a test from changing an input that other tests share outside its process?](https://github.com/kalonji-tools/purba/issues/692) decides which types prove a value immutable.

[A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) names the kind of each breach.
[An error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome) says what each breach stops.
[Each test runs in a process of its own](each-test-runs-in-a-process-of-its-own.md#decision-outcome) says what may change around a test.
