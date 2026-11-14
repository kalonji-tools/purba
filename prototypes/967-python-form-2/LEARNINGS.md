# What prototype 2 of the Python form of an experiment taught (#967). Never merge.

Prototype 2 is a new form, designed from [prototype 1's learnings](https://github.com/kalonji-tools/purba/blob/6591a105b6145c3c4afd40114d8e2085405401c1/prototypes/967-python-form/LEARNINGS.md); [`STRATEGY.md`](STRATEGY.md) gives each rule and the learning it answers.
On the same loguru test, three new blind raters put it at 3.60, against 4.07 for pytest and 3.72 for oxitest in the same round, and 1.97 for prototype 1 in the round before.
[`benchmark/README.md`](benchmark/README.md) holds the numbers, and says why a round compares only with itself: the raters of this round scored pytest and oxitest 0.28 and 0.43 lower than the raters of round 2.
This file sorts what the work found, so that the next design can start from it.

## The five questions of prototype 1, answered

| question | prototype 2's answer | the result |
|---|---|---|
| How does a test of a stateful API stay short without logic? | The act is a straight line of calls. A branch, a loop, `try`, `with`, `assert`, `raise`, a lambda and a nested `def` are refused | 20 items became 20 cases, and 224 lines became 143. The loop of the progressive test became five lines, and all three raters named the list of refusals as a cost |
| What does a step of setup check? | Nothing. It is a line of the act, and only a case must check something | No setup check is left in the port |
| When is a chain replayed, and how many times? | Once for each case; every input taken from it reads that one replay | `examples/ex3_cart.py` shows it. The loguru port needed no chain |
| How does a comparison by type treat a subclass? | A reading must be an instance of the expected value's type; `True` is never `1`, and `1.0` is never `1` | loguru's `Message` compares as a `str`. No rater met a false failure |
| How does a test check code that runs inside `except`? | `handling(exception, function, *args)` raises the exception and calls the function in its `except` | **Failed on fidelity.** All three raters found that the test now writes the exception's message, so `last_line` reads back text the test supplied |

## What held

Each of these held for all three raters, or a rater named it as a strength.

- **One case on one line, named by its variable,** as a typed call of the act: `bang = logged(message="g", format="{message}!").expect(product="g!\n")`. Clarity 4.0, and adding a case took one line, as in pytest and oxitest.
- **A refusal before the run that suggests the fix,** for an input, an assay or a rule: `gives 'mesage', which is no input of logged; did you mean 'message'?`, `line 128 holds an assert: an assay checks, so return the value and expect it in the case`. All three raters found these more helpful than oxitest's errors.
- **A failure screen that points at the case's line,** with expected, reading and inputs. oxitest points at the assert's line, and a case-field mismatch points into oxitest's own code.
- **The type checkers read each case against the act** through a `ParamSpec`: a misspelled input, an input of the wrong type and a `product` of the wrong type, in both ty and pyright. The porter found that a key typo inside a lambda input, `r["levle"]`, is caught too.
- **A fork for each case resets global state with no teardown.** The porter and rater 1 removed the logger reset, and every case still passed.
- **A runner a rater can read:** 83 + 512 non-blank lines, standard library only. Complexity of the implementation 4.0, against 2.0 for pytest and oxitest.
- **A refusal of one case stops only that case,** and the run ends with a count and exits 1. No rater reported a wrong exit status or a lost case.

## What failed

| finding | evidence | the decision it comes from |
|---|---|---|
| **The tester must learn the most of the three versions.** `experiment`, act, case, `product`, assay, `use`, `handling`, and the refused constructs: 14 or more by rater 1's count, about 25 by rater 2's | complexity for the tester 2.3, against 4.0 for pytest and 2.7 for oxitest | the shape of [#963](https://github.com/kalonji-tools/purba/issues/963#issuecomment-6861114993); the straight-line act |
| **The smallest passing test is 8 or 9 lines,** against 6 in pytest and in oxitest: a decorator, a function that returns its product, and a named case with `.expect(product=…)` | all three raters; smallest test 3.0 | an experiment needs a case and an expected value apart from the act |
| **Code inside an `except` loses fidelity:** `handling(ZeroDivisionError("division by zero"), …)` | all three raters; rater 2 changed the message and the check followed it | no `try` in a test (#963 P6) |
| **A wrong-typed input needs a bare `# type: ignore`, which hides every other typo on that line** from both checkers | all three raters; the porter's probe `added(fromat=-1)  # type: ignore` | the case is a typed call |
| **No fixture applies to every case,** so five acts each name `use(Writer)` and `use(fresh_logger)`. Four of the eight lines over 100 characters are these signatures | all three raters | a fixture is a default of the act |
| **A case's refusals come one at a time:** an assay value of the wrong type stays hidden behind a misspelled assay name | rater 1 | the runner stops at the first refusal |
| **One logic construct in an act refuses every case of its experiment,** seven in rater 1's probe | rater 1 | a refusal of the act applies to the experiment |
| **The failure screen cuts a reading at 300 characters,** so the traceback lost its last line, the line under test | rater 3 | the runner |
| **The runner needs `os.fork` and the test's source on disk** | all three raters, not scored | the seal of [#964](https://github.com/kalonji-tools/purba/issues/964#issuecomment-6862174563) |

## What oxitest did that prototype 2 did not

oxitest's port came second in two ratings of three. In all three it scored 5 on fidelity, and the highest clarity or a tie for it. The next design starts from its API.

- **An assert message is the intent of a check,** and the failure screen prints it: "a format function that omits {exception} must not get the traceback appended". Clarity 4.7, the highest of the three versions. Prototype 2 prints an assay's docstring under a failure, but the port wrote none, and no rater named it.
- **A test is a plain function with plain asserts,** and the code inside an `except` stays as the original wrote it, with a real `1 / 0`. Fidelity 5.0 in all three ratings.
- **Its costs, as the raters measured them:** a frozen dataclass for each parametrized test, three of them with one field (142 non-blank lines against 67); a runtime stand-in for a stub-only type, because oxitest calls `get_type_hints()` at collection; one bare `assert` under `strict = "abort"` stops all 22 items, and the message names the rule without the fix; a field of a case that matches no parameter (`colorise`) is caught by neither checker nor the strict mode, and fails inside `oxitest/_bridge/_runners.py`; a teardown that raises is a warning, and the run exits 0; `-vv` names no test.

## What pytest did that neither port did

- The shortest file and the smallest test: 67 non-blank lines, 6 for the smallest test, readability 4.7.
- Runtime errors a rater praised: a misspelled fixture lists the fixtures there are; a wrong value prints a diff.
- Its costs: positional case tuples with ids such as `test_invalid_format[format2]`, no message on any assert, a fixture injected by its name from another file, and 9 errors from both checkers on loguru's own `conftest.py`.

## What the next design must answer

The next prototype improves on oxitest's API, under the strictness that purba expects: the shape of [#963](https://github.com/kalonji-tools/purba/issues/963#issuecomment-6861114993), the seal of [#964](https://github.com/kalonji-tools/purba/issues/964#issuecomment-6862174563), the assays of [#965](https://github.com/kalonji-tools/purba/issues/965#issuecomment-6865490125) and the chains of [#966](https://github.com/kalonji-tools/purba/issues/966#issuecomment-6865928954).

- **How does a case stay named and typed without a dataclass for each test?** oxitest's cases cost a class each; prototype 2's typed call cost none.
- **Where does the intent of a check live,** the message that made oxitest the clearest, when purba compares each reading itself?
- **How does a test check code inside `except`** with the exception that the code raises, and without a `try` that the shape refuses?
- **How does a case pass a wrong-typed input on purpose** without hiding other typos on its line?
- **Which words of purba's shape must a tester learn,** and which can a plain function and a plain value carry? The tester's complexity was the lowest score of both prototypes.
- **How does a fixture reach every case of a file** without an argument on every act, and stay visible where it is used?
- **How does a refusal report every fault of a case at once,** and stop no more cases than the fault reaches?
