# What prototype 3 of the Python form of an experiment taught (#967). Never merge.

Prototype 3 starts from oxitest's API, a plain test function with plain asserts, and holds it to the strictness of map #953 by reading the test's source.
On the same loguru test, three new blind raters put it at 3.74, against 3.84 for pytest and 3.66 for oxitest in the same round.
It is the first prototype above oxitest, and the first within 0.10 of pytest; [`benchmark/README.md`](benchmark/README.md) holds the numbers.
What still costs it the most is the list of rules, which did not shrink when the words did.

## The seven questions of prototype 2, answered

| question | prototype 3's answer | the result |
|---|---|---|
| How does a case stay named and typed without a dataclass for each test? | `name = case(test, input=…)`, typed by the test's `ParamSpec`; the expected value is an input, so it is typed too | **Held.** Each rater's misspelled input and wrong-typed expected value were caught by ty, pyright and the runner. The case lines "scan like a spreadsheet" |
| Where does the intent of a check live? | An optional message on the assert, as in oxitest | **Not tested.** The original has no message, and the porter wrote none. Clarity 4.0, against oxitest's 4.7, whose port added a message to every assert |
| How does a test check code inside `except`? | A `try` of one statement and one `except Type:`; the runner proves the body raised | **Held.** Fidelity of the two exception tests is whole, and all three raters counted the proof as more than the original checks |
| How does a case pass a wrong-typed input on purpose? | The input is declared `object`; the act's line carries a coded ignore | **Held.** No case line carries an ignore; the act line needs two pyright codes and one ty code |
| Which words must a tester learn? | None for a test with no cases; then `case`, `raises`, `warns`, `use` | **Half held.** The smallest test is pytest's, byte for byte (4.7). But complexity for the tester stayed at 2.3, the lowest of the three: raters counted the 13 rules, not the 4 names |
| How does a fixture reach every case of a file? | A fork resets global state; a one-line value is built in the act | **Failed.** A fork resets what a case changes, not what the import made: loguru's default handler. Every test of the port takes `use(Writer)` and `use(fresh_logger)`, and 12 lines are over 100 characters |
| How does a refusal report every fault at once? | The parse collects every fault of a test and of a case | **Held.** Rater 3 ranked prototype 3 first for it, against their own numbers |

## What held

- **A test with no cases is a pytest test.** The smallest test is 6 non-blank lines with no import from the form, for all three raters.
- **Readability rose to pytest's, 4.0.** Prototype 2 had 3.3, and oxitest 3.0.
- **Errors that name the fix, for every fault at once, and stop only the faulty case.** All three raters named them the best of the three versions.
- **A check that runs in a fork, compared by the runner.** `-O` changes nothing; both checks of the exception test run when the first one fails; a wrong type is named (`a bool is not an int`).
- **The map's strictness, as [`STRATEGY.md`](STRATEGY.md#where-the-maps-strictness-is-held) lists it,** held in a form that reads like pytest: a check that reads only an input is refused, a stock is sealed by a sample or a checkpoint, and a chain replays its step once with its checks.

## What failed

| finding | evidence | the decision it comes from |
|---|---|---|
| **The rules, not the words, make the form hard to learn.** `if`, loops, `and`/`or`, comprehensions, `in`, an assert before the end, and any `with` but two | complexity for the tester 2.3 in both prototypes 2 and 3; rater 1 counted 13 rules, and two of their natural tests were refused | #963's "no logic in a test", read as a rule of syntax |
| **A callback's branch is refused as the test's own.** `test_progressive_format`'s formatter holds an `if`, so it moved to a helper file, where nothing reads it; the hint "make each side a case" cannot apply | all three raters; fidelity 4.0 | the no-logic rule walks into every function defined inside the test |
| **No fixture reaches every test of a file** | all three raters; 12 lines over 100 characters | a fork for each case, taken as a reset |
| **The test as written never runs.** The runner parses it, compiles the act again, and evaluates each check | complexity of the implementation 3.0, down from 4.0; 777 + 102 non-blank lines; rater 3: "deep magic" | asserts as assays, without a rewrite of the assert |
| **A syntax rule has holes:** a loop as `list(map(lambda …))` and a branch as arithmetic pass | the porter, confirmed by rater 3 | "from a test's code alone, a tool decides only syntax" (#954) |
| **An `in` check is refused** | rater 1 | a check is `reading == expected`, so the runner compares |

## Decisions of the map this prototype reopens

- **[#963](https://github.com/kalonji-tools/purba/issues/963#issuecomment-6861114993) P6, no `try` in a test.** A `try` whose body must raise the type its `except` names cannot swallow a raise and has one valid path. It kept fidelity whole where prototype 2's `handling(…)` lost it.
- **#963 P5, the assay's name states the question.** A check is named by its source text, `writer.read() == expected`, and an optional message.

## What the next design must answer

- **Which logic is the test's own, and which is a value the test hands to the code?** A format function that the code calls back is test data; a branch in it is not a branch of the test.
- **How does a fixture reach every test of a file, and stay visible?** Neither prototype 2's defaults nor prototype 3's fork answered it.
- **Which rules can go, and which must stay?** The raters scored the length of the list; which of `in`, `is None`, a loop over literal data, keep the shape?
- **Should the runner run the test as written,** with its asserts rewritten as pytest does, instead of compiling a new act?
- **Is a message on a check required, optional, or absent?** The surveys of #954 agree to remove a message on every assertion, and oxitest's messages gave it the clearest port.
