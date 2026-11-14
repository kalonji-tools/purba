# What prototype 4 of the Python form of an experiment taught (#967). Never merge.

Prototype 4 keeps prototype 3's form, and changes four things: one rule for the act, `before_each`, `handling` in place of a `try`, and the test run as written.
On the same loguru test, three new blind raters put it at 3.82, against 4.07 for pytest and 3.63 for oxitest in the same round; [`benchmark/README.md`](benchmark/README.md) holds the numbers.
It widened the lead over oxitest. Its port keeps the original's loop and format function in the test, where prototypes 2 and 3 wrote the loop out and prototype 3 moved the function to a helper file.
Complexity for the tester did not move: 2.3, for the third prototype in a row. The rules sheet grew from 874 words to 1,099, while the rules it holds shrank.

## The five questions of prototype 3, answered

| question | prototype 4's answer | the result |
|---|---|---|
| Which logic is the test's own, and which is a value the test hands to the code? | A function, lambda or class whose only use is as an argument of a call is a value: its body may hold any code but an assert. One the test calls, under any name, or builds, is part of the act | **Held.** `test_progressive_format`'s format function, with its `if`, stays in the test; no rater named it. The porter found two holes (a class the test builds, a function called through another name), fixed before the rating |
| How does a fixture reach every test of a file, and stay visible? | `before_each(fn)`, one line at module level, runs in each case's process | **Half held.** `use(fresh_logger)` left every signature. `writer: Writer = use(Writer)` stayed on each, and 6 lines are still over 100 characters. A probe of the porter's: with no reset at all, the 20 cases still pass, since each case's writer reads only its own sink |
| Which rules can go, and which must stay? | One rule for the act, "it has one path"; a loop over literal data; any one comparison operator; the expected value on either side | **The rules shrank; the score did not move.** Complexity for the tester 2.3, as for prototypes 2 and 3. `FORM.md` grew by a quarter, because each rule now says what it allows as well as what it refuses. Rater 1 called it "28 dense lines of rules"; between them, the raters still met a refusal for a `try`, an `if`, an `and` in an assert and a computed expected value |
| Should the runner run the test as written? | Yes: the last asserts are rewritten at import, as pytest does, and the test runs as one function | **Held for the tester, not for the implementation.** No rater called it "magic". But the runner still parses each test to refuse it, and now rewrites it too: 836 + 145 non-blank lines, against prototype 3's 777 + 102. Complexity of the implementation fell from 3.0 to 2.3 |
| Is a message on a check required, optional, or absent? | Optional, as decided on #967 | **Not tested again.** The original has no message, and the porter wrote none. Clarity rose to 4.3, from the failure screen; oxitest's port, with a message on every assert, scored 4.7 |

And the open item from #967, **checks on the exception object**: `with raises(T) as caught:`, then `caught.value`. Built and shown in `examples/test_5_except.py`; the loguru test does not need it, so no rater met it.

## What held

- **The errors and the failure screen,** for the third round: all three raters named them the best of the three versions, and only the faulty case stops.
- **The smallest test is pytest's, byte for byte,** for all three raters.
- **A callback is the code's, not the test's.** The format function with an `if` stays in the test, and the port has no helper for test logic.
- **`handling` keeps the original's traceback,** once it is a class: a generator added two frames of the form's own code (the porter's probe; `cmprobe/` in the session's scratchpad).
- **The test runs as written.** Called by hand, it is a plain function with Python's asserts; under the runner, `python -O` changes nothing.

## What failed

| finding | evidence | the decision it comes from |
|---|---|---|
| **The rules sheet is the cost, not the rules.** Fewer rules in a longer sheet scored the same | complexity for the tester 2.3 in rounds 3, 4 and 5; `FORM.md` 874 → 1,099 words | the map's strictness written as a sheet a tester reads first |
| **The runner grew.** It reads each test to refuse it, and then rewrites it | complexity of the implementation 2.3; 981 non-blank lines | #963's allow list read from syntax, with pytest's rewrite added beside it |
| **`handling` moves where the log call happens.** loguru records `cases:__exit__:157`, not the test's function and line | rater 3; a probe after the rating: `logger.opt(depth=1).exception` moves it to the test's `with` line, not the original's `except` line | #967: purba supplies `handling`, and a test holds no `try` |
| **A fixture is still on every signature** | `writer: Writer = use(Writer)` on all 8 tests; the signature of `test_format` is 122 characters | a fixture is the default of an input (#965); `before_each` gives no value |
| **Ordinary code is still refused** | a computed expected value (`level + ": x\n"`), an `and` inside an assert, a `try` | #963 P7 (a literal expected value), the one-path rule, #963 P6 |

## What the next design must answer

- **Can a tester learn the form from the runner's refusals, and not from a sheet?** The raters praised the refusals three rounds running, and scored the sheet three rounds running. A sheet of what a tester writes, with the refusals left to the runner, is untried.
- **Can one pass serve both refusal and rewrite?** The runner parses each test twice: once to refuse it, and once to rewrite its asserts.
- **How does a fixture give a value to every test of a file without an input on each signature,** and stay visible? `before_each` gives none.
- **Is the record `handling` leaves acceptable?** A test that reads loguru's `{function}` or `{line}` inside an `except` would read the form's code. Only the two logging libraries of the six scanned run code inside a live `except` in a test (#967 comment of the P5 and P6 rules).
- **Is a computed expected value, such as `level + ": x\n"`, a departure to allow with a reason, or a refusal to keep?** Rater 3 met it while writing a natural test.
