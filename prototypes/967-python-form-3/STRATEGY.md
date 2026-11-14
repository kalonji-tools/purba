# Prototype 3: oxitest's API, held to the map's strictness (#967). Never merge.

Prototype 3 starts from [oxitest's port](https://github.com/kalonji-tools/purba/tree/6591a105b6145c3c4afd40114d8e2085405401c1/prototypes/967-python-form/benchmark/loguru-format/oxitest) of the loguru test: a plain test function, plain asserts with an intent, a real `1 / 0` inside a real `try`.
It answers the seven questions of [prototype 2's `LEARNINGS.md`](https://github.com/kalonji-tools/purba/blob/dc2cbf4a518e195bf5e32d675b12e89736086cd7/prototypes/967-python-form-2/LEARNINGS.md#what-the-next-design-must-answer), and holds the shape that [#963](https://github.com/kalonji-tools/purba/issues/963#issuecomment-6861114993), [#964](https://github.com/kalonji-tools/purba/issues/964#issuecomment-6862174563), [#965](https://github.com/kalonji-tools/purba/issues/965#issuecomment-6865490125) and [#966](https://github.com/kalonji-tools/purba/issues/966#issuecomment-6865928954) decided.
The tester writes what looks like a pytest test; the runner reads its source, and runs it as the shape.
[`FORM.md`](FORM.md) is the rules sheet that the porter and the raters got.

## The strategy in one picture

```text
 def test_format(message: str,                    ← a plain function: its parameters are the inputs
                 fmt: str | FormatFunction,
                 expected: str) -> None:          ← an expected value is an input too, so it is typed
     writer = Writer()                            ┐ the act: a straight line; a value is built in one
     logger.add(writer, format=fmt)               │ line, and a fork for each case resets the logger
     logger.debug(message)                        ┘
     assert writer.read() == expected, "intent"   ← the checks: the asserts at the end. Each one runs
                                                    in a fork of its own, all of them run, the runner
                                                    compares, and the message prints with a failure
 plain = case(test_format, message="a", fmt="{message}", expected="a\n")
 └ name        └ a typed call: ty and pyright read it against test_format's parameters

 try:                                             ← a `try` of one statement and one `except`:
     _ = 1 / 0                                      the runner proves its body raised, so the
 except ZeroDivisionError:                          `except` ran; else the case fails
     logger.exception("Error!")

 with raises(TypeError):                          ← an expected raise, around the last statement
     logger.add(writer, format=fmt)
```

## The seven questions of prototype 2, answered

| question | answer in prototype 3 | what it changes |
|---|---|---|
| How does a case stay named and typed without a dataclass for each test? | `name = case(test, input=…)`: a call typed by the test's `ParamSpec`. The expected value is a parameter of the test, so the checkers type it too | oxitest's frozen dataclass for each test; prototype 2's untyped `expect(**readings: object)` |
| Where does the intent of a check live? | In the assert's message, as in oxitest: `assert reading == expected, "intent"`. It is optional; the failure screen names the check by its source text, and prints the message under it | prototype 2's assay docstring, which no porter wrote |
| How does a test check code inside `except`, without a `try` that the shape refuses? | A `try` of one statement and one `except Type:`, with no `else` or `finally`. The runner adds an `else` that fails the case, so the `except` ran in every case that passes | prototype 2's `handling(exception, …)`, which wrote the message under test itself |
| How does a case pass a wrong-typed input on purpose, without hiding other typos? | The input is declared `object`; the one line of the act that breaks a type carries a coded ignore. The case lines stay fully checked | prototype 2's bare `# type: ignore` on a case line |
| Which words must a tester learn? | For a test with no cases, none: a plain function and plain asserts. Then `case`, `raises`, `warns` and `use`, and the rules of the straight line | prototype 2's `experiment`, assay, `product`, `.expect`, `handling` |
| How does a fixture reach every case without an argument on every act? | A fork for each case resets global state, so no fixture is needed for a reset. A value the test builds in one line is built in the act. `use(factory)` stays, for a stock and for a costly or shared value. *The port showed this half true: a fork resets what a case changes, not what the import made ([`LEARNINGS.md`](LEARNINGS.md))* | prototype 2's `use(Writer)` and `use(fresh_logger)` on five signatures |
| How does a refusal report every fault at once, and stop no more cases than it reaches? | The parse collects every fault of a test, and of a case, and prints them together. A fault of a case stops that case; a fault of the test stops its cases | prototype 2's first fault only |

## Where the map's strictness is held

| decided | held by | built |
|---|---|---|
| #963 the reaction is its inputs and one act, by an allow list | the act is a straight line: no branch, loop, comprehension, `raise`, `global`, `yield` or `await`, inside the test or in a function or lambda defined in it. Prototype 2 had already widened one act to a straight line, on the evidence of prototype 1 | yes |
| #963 P6 no `try` in a test | **reopened here**: a `try` whose body must raise the type its `except` names, or the case fails. One path is valid, so it is no branch, and it cannot swallow a raise it does not name | yes |
| #963 at least one assay, every assay fixed | a test with no assert, no `raises` and no `warns` is refused; the checks are the asserts at the end, and an assert before the end is refused | yes |
| #963 each expected value is a literal in the case | the right side of a check is a literal, an input, or a list, tuple, set or dict of these; a case gives each such input a literal | yes |
| #963 P5 the assay's name states the question; purba prints expected, reading, inputs | the check is named by its source text, `writer.read() == expected`, and its message if it has one | yes |
| #963 PY3 `assert` and `-O` | the runner never runs an assert: it reads the reading and the expected value, and compares them itself | yes |
| #963 PY1 a warning the table does not expect fails | yes, and `warns(…)` expects one | yes |
| #963 PY5 an expected raise is a reading, one statement | `with raises(Type, match=…)` around one statement, the last of the act | yes |
| #963 PY6 two experiments with one name | a test name defined twice in a file, and a case name bound twice, are refused | yes |
| #963 PY8 an error the act loses | an error in a thread or a finaliser fails the case | yes |
| #963 PY4 a stand-in matches the declared type | a fixture's value is checked against the input's annotation, when the annotation can be read at run time | yes |
| #963 P9 public names; P14 a skip carries a reason; the syntax of a departure | — | no |
| #964 a fork seals the product | each check but the last runs in a fork of its own, after the act; a handle that two checks read is refused | yes |
| #965 an assay reads the product, or an input the act can change | a check reads what the act made; a check that reads only an input holding a `str`, number, `bool`, `bytes`, `None` or tuple is refused | yes |
| #965 a stock's fixture offers a sample, a checkpoint, or both | `Samples[T]` and `Checkpoints[T, M]`; a check of a stock runs in place between a checkpoint and a restore; with no checkpoint, two checks of one stock are refused; an abstract stock (a misspelled method) is refused | yes |
| #965 a fingerprint of a stock | — | no |
| #965 when the act raises as expected, the assays of inputs run | the checks after a `with raises(…)` run; a check of a name assigned inside it is refused | yes |
| #966 a chain replays its earlier steps, with their assays, inside one sample of each stock | a case takes `earlier.product`; the earlier case runs again first, with its checks, once for each case, inside the case's own sample or checkpoint; a step that reads differently blocks the case and is named | yes |
| #966 a chain from a case that raised takes an input only | — | no: `.product` of a test that returns nothing is refused, and there is no form for an input yet |

## The trade it makes

The form reads like pytest, so a tester learns less, and the runner must read more.
The runner takes the test's source apart: the statements before the last asserts become a function of their own, and each assert becomes a reading and an expected value that the runner evaluates in a fork.
That costs a parse of every test, and source on disk, as prototype 2 did.
A test that a tester runs by hand, as a plain function, runs its asserts as Python asserts; under the runner, they never run as asserts.
