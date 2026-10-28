# Each test runs in a process of its own

## Context and Problem Statement

The tests of a module share one interpreter, as [a worker loads each module once, and command hands each module once](a-worker-loads-each-module-once.md#decision-outcome) decides.
So a test can fail for what an earlier test left.

| fact | source |
|---|---|
| the text of click's tests showed none of its 3 leaks, because each is behind a call | [the prototype](https://github.com/kalonji-tools/purba/blob/1e0ef96/proto-689/RESULTS.md) |
| click's 1,978 tests ran in 2.4 s in one interpreter, and in 16.0 s with a fork for each test | the same |
| for each test after the first: a fork 0.9 ms, a forkserver with the module preloaded 3.7 ms, a spawn 53 ms | the same |
| the forkserver ignored a preload that raised `ImportError`, and the module body ran 6 times for 5 tests | the same |
| forkserver is "Available on POSIX platforms which support passing file descriptors over Unix pipes", spawn is "Available on POSIX and Windows platforms", and on macOS "The _fork_ start method should be considered unsafe" | [Python's documentation](https://docs.python.org/3/library/multiprocessing.html) |

## Considered Options

- **One interpreter for the tests of a module.** Rejected. click's tests leak through it.
- **The tests whose text shows them isolated share one interpreter.** Rejected. The text of click's tests showed none of its leaks.
- **Each test runs in a process of its own.** Chosen. No change that a test makes in memory reaches another test. A crash ends one test, and not the run.
- **The forkserver of `multiprocessing` loads the module.** Rejected. It ignores a preload that raises `ImportError`, and the module body then runs for each test.
- **A zygote of purba's own loads the module.** Chosen. purba sees a failed load itself, and gives one project error.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**Each [test](../../CONTEXT.md#test) runs in a process of its own.**

- **A zygote of purba's own loads the module once, and forks each test.**
- **A module that fails to load gives one project error.** The zygote then forks no test.
- **On Windows, purba spawns each test.**
- **No record decides how macOS starts each test.**
- **purba adds nothing to a test that the test does not name.**
- **Until purba has its zygote, the tests of a module share one interpreter.**

**Downside:**

- **A tester pays a fork for each test, and a spawn on Windows.**
- **On Windows, a module body with an effect outside the process runs once for each test.** An outcome can then differ by platform.
- **A module whose import starts a thread leaves a thread in the zygote when the zygote forks.** A lock that the thread held stays locked in each forked test.

## Confirmation

No command checks this record yet.
[Each test runs in a process of its own, forked from a zygote that loaded its module once](https://github.com/kalonji-tools/purba/issues/694) builds it, and decides how macOS starts each test.
[Can purba plan a run as Polars plans a lazy query, and reorganize its tests?](https://github.com/kalonji-tools/purba/issues/691) may replace it.
[What does purba do with a task that a test started and did not await?](https://github.com/kalonji-tools/purba/issues/693) decides what purba does with a task that a test leaves.

[A worker loads each module once, and command hands each module once](a-worker-loads-each-module-once.md#decision-outcome) holds how often a module loads until purba has its zygote.
[A test names its inputs by type and returns nothing](a-test-names-its-inputs-by-type-and-returns-nothing.md#decision-outcome) says what a test is.
