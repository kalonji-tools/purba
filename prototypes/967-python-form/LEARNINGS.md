# What prototype 1 of the Python form of an experiment taught (#967). Never merge.

Prototype 1 is `record_strict/`, the form that the grilling of #967 agreed step by step.
On a real test from loguru, three raters put it far below pytest and oxitest: 1.97 against 4.35 and 4.15, on the mean of their weighted scores.
[`benchmark/README.md`](benchmark/README.md) holds the numbers and the test; a later prototype is measured against them, on a branch of its own.
This file sorts what the work found into what held, what failed, and what the tools do, so a new design can start from it.

## What held

Each of these held in both ratings, or a rater named it as a strength.

- **A fork for each case, and for each assay but the last, isolates global state with no teardown.** The loguru port resets loguru's global logger only when it builds the input (`loguru_lab.fresh_logger`), and the fork discards what a case leaves behind. pytest's and oxitest's versions reset it before and after every test. ([#964](https://github.com/kalonji-tools/purba/issues/964#issuecomment-6862174563))
- **A refusal before the run, with a hint, is valued.** All three loguru raters named `gives 'mesage', which is no input; did you mean 'message'?` as a strength.
- **A failure screen of expected, reading and inputs is valued,** and naming both types when only the type differs: `expected 1 (int)`, `reading True (bool)`.
- **Typed inputs catch mistakes in the editor.** Only the forms whose cases a type checker reads as calls caught a misspelled input and an input of the wrong type: a class with `dataclass_transform` fields, a call typed by a `ParamSpec`, and oxitest's frozen dataclass cases. ([OUTPUT.txt](OUTPUT.txt), `ex7_inputs`)
- **Named cases help a reader.** The oxitest port scored the highest clarity of the three versions, for case names such as `unclosed_tag=MarkupCase(fmt="<red>")` and for assert messages that become the failure's explanation.
- **A reading compared in the assay's process sends 16 bytes back.** Sending back a list of 1,000,000 values cost 4.9 to 8.9 MB and 68 to 239 ms.
- **A stock that declares each offer as a base is checked;** a Protocol loses an optional offer, such as a misspelled `fingerprint`, without a word.

## What failed

Each line names where it was found. The first four cost the most on the loguru test.

| finding | evidence | the decision it comes from |
|---|---|---|
| **One call per act turns each call of a stateful API into a step of a chain.** loguru's tests call `add`, then `debug`, then read the writer. The port made 44 cases from 20 items, an 11-step chain for one test, and 174 + 50 lines from 87 + 30. | the loguru port, all three raters | the act is one call ([#963](https://github.com/kalonji-tools/purba/issues/963#issuecomment-6861114993)) |
| **The smallest passing test grew from 5 lines to 22-30,** with two classes, two cases and a chain. | all three loguru raters | one call per act |
| **A step of setup must still check something.** Each `add` step had to expect a handler id that no original test cared about: 11 such checks. | the loguru port; raters 2 and 3 | a case that checks nothing is refused |
| **A literal argument is refused,** so `logger.debug("a")` needs a field for `"a"`. | loguru raters 1 and 2 | no literal in the act (#967 Q2) |
| **A typed `expect` in every experiment ranked last of eight forms,** with 1 of 5 for both complexities and for the smallest test. It became a level. | `benchmark/ratings/forms_rater*` | #967 Q3 |
| **A step that two inputs of one case take is replayed twice,** once for each, without a word. With a global logger, the case then read an empty writer. | the loguru porter's probes | a replayed step builds its own inputs (#967 Q8) |
| **Comparing by type first fails a subclass of `str`.** loguru hands a sink a `Message`, a subclass of `str`: `expected ['a\n'] (list)`, `reading ['a\n'] (list)`, with no reason shown. | loguru rater 2 | same type, then equal (#967 Q5) |
| **"One read" is a rule of syntax.** `"".join(w.written)` was refused and `next(iter(w.written))` was accepted. | the loguru port; rater 2 | an assay is one read |
| **No `try` in a test means no test of `logger.exception`,** which captures the exception being handled. The port took the exception from a fixture instead. | the loguru port; all three raters | no logic in a test ([#963](https://github.com/kalonji-tools/purba/issues/963#issuecomment-6861114993) P6) |
| **A refusal removes every case of its experiment from the screen.** One typo took 50 passing checks to 31, while cases chained to the refused ones still ran. | loguru raters 1, 2 and 3 | a refusal applies to the experiment |
| **A failure leaves out the inputs of earlier steps,** so a failing `Logged` case does not name the format it used, and the screen gives no line number. | loguru raters 1, 2 and 3 | inputs printed per case |
| **The runner is about 1,080 lines,** and uses `__getattribute__`, AST scans of method source, frame inspection and a fork per case and per assay. | loguru raters; `engine.py`, `form_record_strict.py`, `run.py` | the shape as a whole |

These are faults of the throwaway runner, not of a decision: it exits 0 when a case fails or is refused, prints no count, and `run.py measure` fails with an `IndexError` when one form is run alone.

## What the type checkers do

Each line was probed with ty 0.0.87 and pyright 1.1.411, and both agree unless the line says otherwise.

- **A base class marked `dataclass_transform` gives a typed constructor,** so a misspelled or mistyped field is an error. With pydantic's `BaseModel` as the base, ty saw 1 of 3 such mistakes.
- **A `ParamSpec` types a call by the parameters of another function,** and `Concatenate[Any, P]` skips its `self`.
- **pyright reports a function in a class with no `self`:** 54 diagnostics in five files. It exempts only a literal `@staticmethod`; a decorator typed to return `staticmethod` is still reported. ty reports nothing.
- **A typed method cannot override an untyped one in a shared base.** `def expect(self, **values: Any)` in the base and `def expect(self, *, cents: int)` in a subclass is an invalid override in both. A base attribute `expect: Callable[..., Any]` is an invalid override in ty only. Two bases, one with the loose method and one without, pass both.
- **ty refuses a method whose body is `...`** when it is not abstract and returns `Self`, even under a decorator; `return self` passes both.
- **ty ignores `# type: ignore[code]`.** A bare `# type: ignore` silences a line in both.
- **Both accept a `bool` where an `int` is declared.** So only a run-time comparison by type catches `True` for `1`.
- **Neither reports a module-level name bound twice,** so a second case of one name loses the first without a word.
- **A base whose methods a subclass overrides must take parameters by position only** (`def sample(self, value: T, /)`), or a stock that names them `db` is an invalid override.
- **A type that exists only in a library's stubs raises `NameError` at run time** wherever the runner reads annotations: loguru's `Logger` and `FormatFunction`. `Experiment["Logger"]`, or a stand-in under `else` of `TYPE_CHECKING`, works around it. oxitest's collection hit the same.
- **A model that pydantic builds at run time is invisible to both checkers.** Its strict check of expected values gave the same verdicts as a 17-line check over the literal types, in 5 cases of 5, and its import took 116 ms.

## What the other two leave open

The raters found these in pytest and oxitest. A new design can aim at them.

- **pytest:** cases are untyped tuples, so ty passes a malformed case; ids such as `test_format[e-<lambda>-e]` and `test_invalid_format[format2]` say little; a fixture is injected by its name; both checkers report 9 errors on loguru's own `conftest.py`.
- **oxitest:** each parametrized test needs a dataclass of its own, which made the port 186 + 33 lines; a teardown error is only a warning, and the run exits 0; with `-vv` the console still names no test; a type that exists only in stubs breaks collection.

## What a new design must answer

- **How does a test of a stateful API, a sequence of calls on one object, stay short** without bringing logic back into the test? One call per act cost the most here.
- **What does a step of setup check,** when a case that checks nothing is refused?
- **When is a chain replayed, and how many times,** when two inputs come from one step?
- **How does a comparison by type treat a subclass,** such as loguru's `Message` for a `str`?
- **How does a test check code that runs inside `except`,** such as `logger.exception`?
