# Prototype 2: a new API strategy for the Python form of an experiment (#967). Never merge.

Prototype 1 (`record_strict/` on `proto/967-python-form-of-an-experiment`) scored 1.97 against pytest's 4.35 and oxitest's 4.15 on a real loguru test.
Its [`LEARNINGS.md`](https://github.com/kalonji-tools/purba/blob/6591a105b6145c3c4afd40114d8e2085405401c1/prototypes/967-python-form/LEARNINGS.md) names what held, what failed, and five questions for a new design.
This strategy starts from those, not from `record_strict/`. [`FORM.md`](FORM.md) is the rules sheet that the porter and the raters got.

## The strategy in one picture

```text
 @experiment                        ← an experiment is a function
 def formatted(message: str,        ← its parameters are the inputs
               format: str,
               writer: Writer = use(Writer)) -> str:   ← a fixture is a default
     logger.add(writer, format=format)                 ┐ the act: a straight line of calls,
     logger.debug(message)                             │ literals and nested calls allowed;
     return writer.read()                              ┘ no if · loop · try · with · assert

 @formatted.assay                   ← an assay: one `return` of the product or one input
 def first_line(product: str) -> str:
     return product.splitlines()[0]

 plain = formatted(message="a", format="{message}").expect(product="a\n")
 └ name  └ a typed call: ty and pyright see a typo or a wrong type, product included
```

## Each learning, and the answer here

| learning (prototype 1) | answer in prototype 2 | what it changes |
|---|---|---|
| One call per act turned each call of a stateful API into a step of a chain: 44 cases from 20 items | **The act is a straight line of statements.** Any calls, in order; no branch, loop, `try`, `with`, `assert`, `raise`, lambda or nested `def` | #963's "the act is one statement" and its allow list |
| The smallest test grew from 5 lines to 22-30 | A function, its body, and one case; [`benchmark/`](benchmark/README.md) has the raters' count | the class, the abstract `act`, the two levels |
| A step of setup had to check something (11 handler-id checks) | **Setup is a line of the act.** Only a case must check something | a chain step for each call |
| A literal argument was refused | **Literals and nested calls are allowed in the act.** Logic is a branch or a loop, not a value | #967 Q2 |
| A typed `expect` in every experiment ranked last of eight forms | **The case is a call typed by the act's `ParamSpec`**, the best-rated of the eight forms (`call/`, 4.11). `expect(product=…)` is typed by the act's return; an assay's expected value is checked against its return type before the run | #967 Q3's two levels |
| A step that two inputs take was replayed twice | **An earlier case replays once per case;** every input taken from it reads that one replay (`examples/ex3_cart.py`, `both_inputs_one_cart`) | #967 Q8 |
| Type first, then value, failed loguru's `Message`, a subclass of `str` | **A reading must be an instance of the expected type**, except that `True` is never `1` and `1.0` is never `1`; items compared one by one; a type failure names both types | #967 Q5 |
| "One read" was a rule of syntax (`"".join` refused, `next(iter())` accepted) | **An assay is one `return`, under the same no-logic rule as the act.** `splitlines()[0]` and `"".join(…)` pass | "an assay is one read" |
| No `try` meant no test of `logger.exception` | **`handling(exception, function, *args)`** raises the exception and calls the function inside its `except` | #963 P6 for this one case |
| A refusal removed every case of its experiment | **A refusal of a case stops only that case;** a refusal of an act or an assay stops its experiment | "a refusal applies to the experiment" |
| A failure left out the inputs of earlier steps, and gave no line number | Each failure prints `file:line` of the case, the inputs, the assay's docstring if any, and the output the case printed | — |
| The runner exited 0 on a failure, printed no count, and ran only listed files | A count line; exit 1 unless every case passed; a file or folder as the argument | — |
| A stub-only type (`Logger`) raised `NameError` where the runner read annotations | The runner reads annotations as text, and evaluates only builtins; a stub-only name is not read | — |
| A case name bound twice lost a case without a word | Refused: "line N binds 'x' again, so this case would be lost" | — |

## What held, and stays

- A fork for each case, and for each assay but the last ([#964](https://github.com/kalonji-tools/purba/issues/964#issuecomment-6862174563)). A case resets global state, such as loguru's logger, only by building its inputs; the fork discards the rest.
- A refusal before the run, with a hint: `gives 'mesage', which is no input of formatted; did you mean 'message'?`.
- A failure screen of expected, reading and inputs, with both types when only the type differs.
- Named cases, and an expected value that is a literal (a computed one is refused).
- A reading compared in the assay's process: only the verdict and its two texts cross back.
- An assay reads the product or one input ([#965](https://github.com/kalonji-tools/purba/issues/965#issuecomment-6865490125)); a case that raises as expected still runs the assays that read an input.

## What prototype 2 does not build

These are decided or kept in mind on the map, and prototype 1's runner held some of them. Prototype 2 leaves them out, so its runner is smaller for that reason too.

- A stock and its offers (sample, checkpoint, fingerprint), from #965.
- The public-name rule, a skip with a reason, and the syntax of a suppression, from #963.
- A refusal of a product that is a file, a socket or a pipe read by two assays, from #964.
- A replay's own sample or checkpoint, from #966.

## What the type checkers see

ty 0.0.87 and pyright 1.1.411 both report each of the three mistakes in `examples/ex7_typed.py`: a misspelled input, an input of the wrong type, and a `product` of the wrong type. Neither sees a misspelled assay name or an assay's expected value of the wrong type (`expect(**readings: object)`); the runner refuses both before the run, with a hint. Both report nothing else on `experiment.py`, `run.py` and `examples/`. `CHECKERS.txt` holds the run.
