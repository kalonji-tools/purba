# PROTOTYPE results: can the text of a test show that it is isolated?

Throwaway. This branch never merges. `run.sh` reproduces every number below, and
`runs/` holds the outputs of the run this file reads. The detector's raw output stays in
the work directory, because click's test ids hold words that the typos hook refuses.

## The question

The person's idea: a test that is completely isolated can be seen from its signature.
purba then runs those tests together in one interpreter, and gives only the rest a
process of its own (the "beaker").

## The answer

**No level of the text both finds the leaks and leaves enough tests to bundle.**
A leak lives where the code under test changes state, and a test reaches that code
through a call. The signature never shows a call, and the body shows only the call,
not what it changes.

```
                  says isolated    of those, leak    verdict
 L0 signature     490 of 490       3                 sees nothing: every leak passes
 L1 + body        488 of 490       3                 sees only a direct write
 L2 + calls         7 of 490       0                 sound, and bundles 1.4%
```

The 3 leaks in click, each behind a call:

```
 test_arguments.py::test_nargs_star    click.globals._local   click makes its context stack on first use
 test_info_dict.py::test_command       HELLO_COMMAND          to_info_dict caches a help Option on a
                                                              module-level Command that cases share
 test_arguments.py::test_file_atomics  random state           click's atomic file calls random.randrange
```

None of them changes an outcome: click passes forward, reversed and in 3 shuffles.

## The control

`planted/` holds 9 tests whose leaks are known before the run.

```
 test                 truth                text (L1)      detector
 test_p_clean         clean                isolated       clean
 test_p_global_dict   leaks CONFIG         ✓ sees it      ✓
 test_p_env           leaks os.environ     ✓ sees it      ✓
 test_p_cwd           leaks cwd            ✓ sees it      ✓
 test_p_helper        leaks SEEN           ✗ a call       ✓
 test_p_env_via_call  leaks os.environ     ✗ a call       ✓
 test_p_click_context leaks click's stack  ✗ a call       ✓
 test_p_lru_cache     leaks the cache      ✗ a call       ✗ state inside C
 test_p_monkeypatch   clean (restored)     isolated       clean
```

## One test in each beaker, on a toy module

`runs/bench.txt`, a module with no imports and one that imports asyncio, json and 8 more.

```
 call the test                   0.1 µs   1.1 µs
 re-import the module            47 µs    56 µs
 os.fork() from a warm parent    1.4 ms   1.9 ms
 a fresh interpreter + import    18 ms    96 ms
 multiprocessing spawn           48 ms    103 ms
```

## The price

click, 1,978 items, CPython 3.12.14, Linux x86_64, 22 cores. Each row ran 3 times
(`runs/fork.txt`).

```
 no beaker (one interpreter)           2.4 s
 a fork for every test                16.0 s    ~6.9 ms a test
 fork what L1 does not clear           2.4 s    2 forked, 1,976 bundled
 fork what L2 does not clear          13.9 s    1,752 forked, 226 bundled
```

A fork in a real runner costs about 3.6 times the toy fork of `bench/` (1.9 ms):

```
 fork to child running      1.8 ms
 the test, in the child     3.4 ms    in the parent, 0.13 ms (runs/durations.txt);
                                      copy-on-write is the likely cause, not measured
 exit and wait              the rest of 6.9 ms
```

An earlier run on the same machine, under more load, measured each fork row at about
twice these numbers. The order of the rows did not change.
The detector itself costs about 21 ms a snapshot, two a test, in Python: more than a fork.

## What it means for the beaker

```
 bundle what the text PROVES isolated     sound, bundles 1.4%: a fork for nearly every test
 bundle what the text does NOT show       bundles 99.6%, misses every leak behind a call
   leaking
 diff the state around every test         saw 6 of 7 planted leaks, costs more than a fork
```

## A process for each test, with the module body run once per module

`bench/forkserver.py`, `runs/forkserver.txt`. The module imports asyncio, json and 8
more, and its body appends one `x` to a file each time it runs.

```
                           first test    each later    module body ran
 fork from the parent       28 ms         0.9 ms       1 time in 200 tests
 forkserver, preloaded      53 ms         3.7 ms       1 time in 200 tests
 forkserver, no preload     55 ms          32 ms       100 times in 100 tests
 spawn                      55 ms          53 ms       50 times in 50 tests
```

A module that fails to load, 5 tests:

```
                     raises RuntimeError                  raises ImportError
 fork                the parent sees it once              the parent sees it once
 forkserver          the server dies; the parent gets     the server ignores it, and each
                     EOFError: unexpected EOF             child loads again: 6 runs in 5 tests
 spawn               each child: 5 runs                   each child: 5 runs
```

[Python's documentation](https://docs.python.org/3/library/multiprocessing.html) says
forkserver is "Available on POSIX platforms which support passing file descriptors over
Unix pipes such as Linux", and that spawn is "Available on POSIX and Windows platforms".
`set_forkserver_preload` ignores an `ImportError` by default, and takes `on_error` from
3.15.

## Not measured

- A second corpus. click's tests are unusually clean.
- macOS, Windows, CPython 3.14 and 3.14t. Forkserver on macOS: the documentation does not name it.
- A parent the size of a numpy or pandas project.
- A failure re-run from a clean process, to tell an order-dependent failure apart.

## Notes

- pytest-forked 1.7.5 fails every test on pytest 9.2 (the child exits 3), so
  `forkbeaker.py` forks on its own.
- `classify.py` is Python's `ast`, not ruff's. It reads what purba's parsing could read.
