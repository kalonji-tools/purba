# Prototype 4: prototype 3, with one path and the test run as written (#967). Never merge.

Prototype 4 starts from [prototype 3](https://github.com/kalonji-tools/purba/tree/17fb9f6eb33bfb1e80489fcf1216e4715250aa5c/prototypes/967-python-form-3): a plain test function, its checks the asserts at its end, its cases typed calls.
It takes the two rules decided after prototype 3 ([#967 comment](https://github.com/kalonji-tools/purba/issues/967#issuecomment-6877824512)): a test holds no `try`, and purba supplies `handling`; a check is named by its own text, and a message is optional.
It answers the five questions of [prototype 3's `LEARNINGS.md`](https://github.com/kalonji-tools/purba/blob/17fb9f6eb33bfb1e80489fcf1216e4715250aa5c/prototypes/967-python-form-3/LEARNINGS.md#what-the-next-design-must-answer).
[`FORM.md`](FORM.md) is the rules sheet that the porter and the raters got.

## The strategy in one picture

```text
 before_each(reset_logger)                         ← once in the file: runs at the start of each case

 def test_progressive_format() -> None:
     def formatter(record: Record) -> str:          ┐ handed to the code, so a value: its `if` is
         if "noend" not in record["extra"]: …       ┘ the code's path, not the test's
     logger.add(writer, format=formatter)           ┐ the act: one path. A loop over literal data
     for _ in range(5):                             │ runs the same rounds every time
         logger.opt(raw=True).debug(".")            ┘
     assert writer.read() == "…"                    ← a check: one comparison, any operator; the
                                                      runner rewrote it at import into a call that
                                                      forks it, as pytest rewrites an assert

 with handling(ZeroDivisionError, logger.exception, "Error!"):   ← code inside an `except`: no `try`
     _ = 1 / 0

 with raises(ValueError) as caught:                 ← an expected raise; the checks may read
     check_age(12)                                    `caught.value`
 assert str(caught.value) == "too young"
```

## The five questions of prototype 3, answered

| question | prototype 4's answer | what it changes |
|---|---|---|
| Which logic is the test's own, and which is a value the test hands to the code? | A function, lambda or class that the test defines and hands to the code is a value: its body may hold any code but an assert. A function the test calls itself, or hands to `map` or `filter`, is part of the act | prototype 3 refused a branch in any function defined in a test, so the port moved its format function to a helper file |
| How does a fixture reach every test of a file, and stay visible? | `before_each(fn)`, one line at module level, runs `fn()` at the start of each case of that file, in the case's own process. Nothing runs after: the process ends | prototype 3's `use(fresh_logger)` on every signature, and 12 lines over 100 characters |
| Which rules can go, and which must stay? | One rule, "the act has one path", replaces the list of refused constructs. A loop over literal data has one path, so it stays. A check takes any one comparison operator, and the expected value may stand on either side. The rules that go: `in` refused, the reading on the left, any loop refused, a branch in a callback refused, `raises … as` refused | prototype 3's 13 rules; complexity for the tester 2.3 in two rounds |
| Should the runner run the test as written? | Yes. The runner imports a test file with each test's last asserts rewritten as a call to a check, as pytest rewrites an assert. The test runs as one function; at each check the runner forks, except at the last | prototype 3 parsed the test, compiled its act as a new function, and evaluated each check apart from it |
| Is a message on a check required, optional, or absent? | Optional, as decided on #967: a failure names the check by its text, and prints the message under it | — |

And the open item of the same comment, **checks on the exception object**: `with raises(T) as caught:` gives `caught.value`, and the checks after it may read it.

## Where the map's strictness is held

| decided | held by | built |
|---|---|---|
| #963 the reaction is its inputs and one act, by an allow list | the act has one path: no branch, `while`, `try` or `raise`; a loop or a comprehension runs over literal data with no `if`; a function the test calls itself is held to the same | yes |
| #963 P6 no `try` in a test; #967 purba supplies `handling` | a `try` is refused; `with handling(T, call, *args):` fails the case unless the statement raises `T`, and runs `call(*args)` in its own `except` | yes |
| #963 at least one assay, every assay fixed | a test with no check, no `raises`, no `warns` and no `handling` is refused; the checks are the asserts at the end, and an assert before the end is refused | yes |
| #963 each expected value is a literal in the case | one side of a check is a literal, an input, or a list, tuple, set or dict of these; a case gives each such input a literal | yes |
| #963 P5 as changed on #967: a check is named by its text, the message optional | a failure prints the check's source text, and the message under it | yes |
| #963 PY3 `assert` and `-O` | the runner rewrites each check before Python compiles it, so `-O` has no assert to drop | yes |
| #963 PY1 a warning the table does not expect fails | yes, and `warns(…)` expects one | yes |
| #963 PY5 an expected raise is a reading, one statement | `with raises(Type, match=…)` around one statement, the last of the act | yes |
| #963 PY6 two experiments with one name | a test name defined twice in a file, and a case name bound twice, are refused | yes |
| #963 PY8 an error the act loses | an error in a thread or a finaliser fails the case | yes |
| #963 PY4 a stand-in matches the declared type | a fixture's value is checked against the input's annotation, when the annotation can be read at run time | yes |
| #963 P9 public names; P14 a skip carries a reason; the syntax of a departure | — | no |
| #964 a fork seals the product | each check but the last runs in a fork of the case's process, where the test wrote it; a handle that two checks read is refused | yes |
| #965 an assay reads the product, or an input the act can change | a check that reads only an input holding a `str`, number, `bool`, `bytes`, `None` or tuple is refused | yes |
| #965 a stock's fixture offers a sample, a checkpoint, or both | as prototype 3 | yes |
| #965 a fingerprint of a stock | — | no |
| #965 when the act raises as expected, the assays of inputs run | the checks after a `with raises(…)` run, and may read `caught.value`; a check of a name assigned inside it is refused | yes |
| #966 a chain replays its earlier steps, with their assays | as prototype 3: the earlier test runs again, as written, and a check that reads differently blocks the later case | yes |
| #966 a chain from a case that raised takes an input only | — | no |

## The trade it makes

A tester learns one rule for the act, one for a check, and three `with` blocks.
The runner still reads each test's source to refuse what breaks the form; what changed is how it runs a test: as written, with its asserts rewritten.
`before_each` is a step that no input names, so a reader of a test must look at the top of its file for it, as a reader of a pytest test looks for an autouse fixture in `conftest.py`. Here it sits in the same file.
`before_each` runs once in each case's process, before the steps of a chain replay; a chain whose steps change the same global state is not tried here.
