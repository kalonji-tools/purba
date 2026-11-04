# How a signal reaches a test that purba runs from Rust

This file answers [How does a signal reach a test that purba runs from Rust, and where can purba see it?](https://github.com/kalonji-tools/purba/issues/767), a child of [Wayfinder: purba's interrupt handling](https://github.com/kalonji-tools/purba/issues/765).
It is never merged.

## Answer

A Ctrl-C does not interrupt purba where it arrives. The kernel runs CPython's C handler, which only sets a flag.
CPython turns the flag into a `KeyboardInterrupt` later, on the main thread, at the next check of the interpreter loop or at the next `PyErr_CheckSignals`.
purba's Rust code never makes that check, so a Ctrl-C that arrives while Rust runs waits for the next Python code.
**The hypothesis holds.** A Ctrl-C that arrived while purba wrote the line of `test_one` raised `KeyboardInterrupt` in `test_two`, before its first line ran, and purba reported `test_two raised: KeyboardInterrupt:`.
After the last test, the same Ctrl-C ends purba with a traceback and exit 130, after the count line that promised exit 4.
A Ctrl-C can also vanish: when it arrives while purba calls a test's exception `__str__`, PyO3 drops the `KeyboardInterrupt`, and the line reads `<exception str() failed>`.
Today purba cannot tell a signal from a raise. The line of a terminal's Ctrl-C, of `raise KeyboardInterrupt`, of `os.kill(os.getpid(), SIGINT)`, of `signal.raise_signal` and of `_thread.interrupt_main()` is the same.
purba can tell them apart only with a handler of its own: a handler that counts each signal lets purba compare the count before and after each call.
A C-level handler with `SA_SIGINFO` can tell more: the kernel marks a terminal's Ctrl-C `SI_KERNEL`, and a test's own `os.kill` `SI_USER` with the test's pid.
In the planned zygote, a terminal's Ctrl-C reaches every process in purba's process group: purba, the zygote and the test child. A `kill <pid>` reaches only the one process.

| question | short answer | tier |
|---|---|---|
| 1. Where a SIGINT becomes `KeyboardInterrupt` | C handler sets a flag; the main thread runs `default_int_handler` at the next eval-breaker check (`RESUME`, `JUMP_BACKWARD`, after `CALL`) or `PyErr_CheckSignals`. SIGTERM has no handler: `SIG_DFL` | [A] S1–S9, M1 |
| 2. How PyO3 code sees a pending signal | only through `py.check_signals()`, which calls `PyErr_CheckSignals`. PyO3 calls it nowhere else. Holding the GIL delays the handler. `detach` does not run it either | [A] S10–S12 |
| 3. The hypothesis | **holds** on CPython 3.12.14 and 3.14.4, with `kill -INT -- -<pid>` and with a real pty's `^C` | [A] M2–M5 |
| 4. Tester's code | own handler: purba sees nothing, and the handler stays for later tests. `except BaseException`: purba sees `passed`. A thread: the main thread gets the `KeyboardInterrupt`, the thread runs on into later tests | [A] M6 |
| 5. Forked child | a terminal's Ctrl-C reaches parent, zygote and test child; `kill <pid>` reaches one; a child in its own group escapes the terminal | [A] M7 |
| 6. Windows | the console runs each handler on a new thread; Ctrl-C reaches every process on the console; Ctrl-Break has no Python handler by default | [A] docs and source (W1–W7); [C] not run |
| 7. Where purba can tell signal from raise | nowhere today. With a handler of its own: at the return of each call, and between tests. With `SA_SIGINFO`: also who sent it | [A] M6, M8; design points are reasoning |

## 1. CPython: from SIGINT to `KeyboardInterrupt`

```text
 kernel          C handler (any thread,          main thread, later
 delivers   -->  async-signal context)      -->  at an eval-breaker check
 SIGINT          trip_signal():                  _Py_HandlePending()
                   Handlers[SIGINT].tripped=1      handle_signals()
                   is_tripped=1                    _PyErr_CheckSignalsTstate()
                   eval_breaker bit on               calls the Python handler:
                   the MAIN thread state             default_int_handler
                   wakeup fd byte                    -> raise KeyboardInterrupt
```

| step | what CPython does | source (v3.14.10) | tier |
|---|---|---|---|
| start | Installs `signal_handler` for SIGINT, and `default_int_handler` as its Python handler, only when SIGINT was `SIG_DFL` at start. Ignores SIGPIPE and SIGXFSZ. Installs nothing for SIGTERM | [signalmodule.c L1580-L1614](https://github.com/python/cpython/blob/v3.14.10/Modules/signalmodule.c#L1580-L1614), [L1924-L1942](https://github.com/python/cpython/blob/v3.14.10/Modules/signalmodule.c#L1924-L1942); docs: "SIGINT is translated into a KeyboardInterrupt exception if the parent process has not changed it" ([signal.rst L17-L22](https://github.com/python/cpython/blob/v3.14.10/Doc/library/signal.rst?plain=1#L17-L22)) | [A] S1 |
| start, measured | `getsignal`: SIGINT `default_int_handler`, SIGTERM `0` (`SIG_DFL`), SIGPIPE `1` (`SIG_IGN`) on 3.12.14 and 3.14.4 | M1 | [A] |
| C handler | `sigaction` with `SA_ONSTACK` only, so no `SA_RESTART`: a blocked syscall returns `EINTR` or a partial count | [pylifecycle.c L3610-L3635](https://github.com/python/cpython/blob/v3.14.10/Python/pylifecycle.c#L3610-L3635) | [A] S2 |
| C handler | `trip_signal` sets `Handlers[sig].tripped`, then `is_tripped`, then the eval-breaker bit, then writes the wakeup fd. It runs no Python | [signalmodule.c L274-L379](https://github.com/python/cpython/blob/v3.14.10/Modules/signalmodule.c#L274-L379) | [A] S1 |
| which thread | the eval-breaker bit goes to the main thread state only: `_Py_set_eval_breaker_bit(_PyRuntime.main_tstate, ...)` | [ceval_gil.c L663-L666](https://github.com/python/cpython/blob/v3.14.10/Python/ceval_gil.c#L663-L666) | [A] S3 |
| which thread | handlers run only when `_Py_IsMainThread() && _Py_IsMainInterpreter(interp)` | [pycore_pystate.h L81-L86](https://github.com/python/cpython/blob/v3.14.10/Include/internal/pycore_pystate.h#L81-L86), [ceval_gil.c L824-L837](https://github.com/python/cpython/blob/v3.14.10/Python/ceval_gil.c#L824-L837); docs: "always executed in the main Python thread of the main interpreter" ([signal.rst L64-L69](https://github.com/python/cpython/blob/v3.14.10/Doc/library/signal.rst?plain=1#L64-L69)) | [A] S3, S4 |
| when | `_CHECK_PERIODIC` reads the eval breaker and calls `_Py_HandlePending`. It is part of `RESUME` (each function entry, not after a `yield from`), `JUMP_BACKWARD` (each loop turn) and `CALL` (after the callee returns) | [bytecodes.c L155-L173](https://github.com/python/cpython/blob/v3.14.10/Python/bytecodes.c#L155-L173), [L224-L228](https://github.com/python/cpython/blob/v3.14.10/Python/bytecodes.c#L224-L228), [L2935-L2939](https://github.com/python/cpython/blob/v3.14.10/Python/bytecodes.c#L2935-L2939), [L3844](https://github.com/python/cpython/blob/v3.14.10/Python/bytecodes.c#L3844), [ceval_gil.c L1357-L1375](https://github.com/python/cpython/blob/v3.14.10/Python/ceval_gil.c#L1357-L1375) | [A] S5 |
| when, in C | C code calls `PyErr_CheckSignals` itself. `time.sleep` does on `EINTR`: "sleep was interrupted by SIGINT" | [timemodule.c L2278-L2291](https://github.com/python/cpython/blob/v3.14.10/Modules/timemodule.c#L2278-L2291), [signalmodule.c L1772-L1796](https://github.com/python/cpython/blob/v3.14.10/Modules/signalmodule.c#L1772-L1796); docs: "should be called by long-running C code" ([exceptions.rst L691-L716](https://github.com/python/cpython/blob/v3.14.10/Doc/c-api/exceptions.rst?plain=1#L691-L716)) | [A] S6 |
| the raise | the Python handler gets `(signum, frame)` and nothing else. `default_int_handler` does `PyErr_SetNone(PyExc_KeyboardInterrupt)` | [signalmodule.c L232-L237](https://github.com/python/cpython/blob/v3.14.10/Modules/signalmodule.c#L232-L237), [L1801-L1885](https://github.com/python/cpython/blob/v3.14.10/Modules/signalmodule.c#L1801-L1885) | [A] S1 |
| at exit | an uncaught `KeyboardInterrupt` sets `unhandled_keyboard_interrupt`; `Py_RunMain` then resets SIGINT to `SIG_DFL` and kills itself with SIGINT, so a shell sees 130 | [pythonrun.c L630-L636](https://github.com/python/cpython/blob/v3.14.10/Python/pythonrun.c#L630-L636), [main.c L795-L817](https://github.com/python/cpython/blob/v3.14.10/Modules/main.c#L795-L817), [L834-L855](https://github.com/python/cpython/blob/v3.14.10/Modules/main.c#L834-L855) | [A] S7, M3 |
| simulated | `PyErr_SetInterruptEx` (behind `_thread.interrupt_main`) calls the same `trip_signal`, with no OS signal | [signalmodule.c L1902-L1915](https://github.com/python/cpython/blob/v3.14.10/Modules/signalmodule.c#L1902-L1915), [_thread.rst L61-L80](https://github.com/python/cpython/blob/v3.14.10/Doc/library/_thread.rst?plain=1#L61-L80) | [A] S1, S8 |
| SIGTERM | no Python handler, so the default action ends the process: no `finally`, no line. The tester's view: exit 143, empty stdout | M1; [What purba does on an interrupt today](https://github.com/kalonji-tools/purba/issues/765#issuecomment-6626393301) | [A] M1 for `SIG_DFL`; [B] for the exit |
| inherited `SIG_IGN` | a shell without job control starts a background job with SIGINT ignored (POSIX [2.12](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html#tag_19_12)). CPython then keeps `SIG_IGN`, and a Ctrl-C does nothing to purba | M9 | [A] |

## 2. PyO3 0.29.2: what Rust code sees

purba's `Cargo.lock` pins `pyo3` 0.29.2. Each file cited below is byte-identical between the crate in the cargo registry and the `v0.29.2` tag (S10).

| question | answer | source | tier |
|---|---|---|---|
| How does Rust see a pending signal? | `Python::check_signals` calls `PyErr_CheckSignals` and turns `-1` into the `PyErr`, usually `KeyboardInterrupt` | [marker.rs L735-L775](https://github.com/PyO3/pyo3/blob/v0.29.2/src/marker.rs#L735-L775) | [A] S10 |
| From a thread that is not the main thread? | "it does nothing yet still returns `Ok(())`" | [marker.rs L768-L769](https://github.com/PyO3/pyo3/blob/v0.29.2/src/marker.rs#L768-L769) | [A] S10 |
| Does PyO3 check signals by itself? | no: in `src/`, only `marker.rs` names `PyErr_CheckSignals`. `pyo3-ffi` also binds `PyErr_SetInterrupt(Ex)` and `PyOS_InterruptOccurred` | [pyerrors.rs L353-L358](https://github.com/PyO3/pyo3/blob/v0.29.2/pyo3-ffi/src/pyerrors.rs#L353-L358) | [A] S11 |
| Rust holds the GIL a long time | the C handler still sets the flag at once. No Python handler runs until Rust calls `check_signals`, or calls into Python code that reaches an eval-breaker check. Rust's own syscalls see `EINTR`; `write_all` retries it, and continues after a partial write | [core/io/write.rs L180-L225 at 1.99.1](https://github.com/rust-lang/rust/blob/1.99.1/library/core/src/io/write.rs#L180-L225). The wheel was built with the pinned nightly, not 1.99.1; M5 shows the partial write and the rest of it on that wheel | [A] S12, M5 |
| Rust releases the GIL (`Python::detach`) | `detach` suspends the thread state and runs the closure. Other Python threads run, but the eval-breaker bit is on the main thread state, so none of them runs the handler. The main thread runs it after it reattaches and reaches a check | [marker.rs L501-L573](https://github.com/PyO3/pyo3/blob/v0.29.2/src/marker.rs#L501-L573), [ceval_gil.c L663-L666](https://github.com/python/cpython/blob/v3.14.10/Python/ceval_gil.c#L663-L666) | [A] S3, S10 |
| An error that PyO3 drops | `Display for PyErr` calls `value.str()` and, on `Err`, writes `<exception str() failed>`. The `Err` is dropped, so a `KeyboardInterrupt` raised inside `__str__` disappears | [err/mod.rs L691-L703](https://github.com/PyO3/pyo3/blob/v0.29.2/src/err/mod.rs#L691-L703) | [A] S10, M8 |
| What purba calls | no `check_signals`, no `detach`, no `signal` call anywhere in `crates/` at 9051ab2 | `grep` in M0 | [A] |

## 3. The hypothesis: a Ctrl-C between two tests charges the next test

**It holds.** Each run below used a wheel built from `main` at `79288dc`; between `79288dc` and `9051ab2` only `crates/execution/tests/run.rs` changes under `crates/` (M0).

Where purba runs Rust between two tests, at 9051ab2:

```text
 _main (purba/src/lib.rs L29)                     main thread, GIL held throughout
  └ command::run ─ dispatch::hand (command L94)
       └ execution::attach ─ Python::attach        (execution L224-L233)
           for each test (dispatch L84-L88):
             worker.run ─ call:
               module.getattr(function)            (execution L304)   C, no bytecode
               test.is_callable()                  (execution L308)   C, no bytecode
               watch(|| test.call0())              (execution L315)   ← test's bytecode
                                                                         RESUME checks here
               err.to_string() for a raise         (execution L329-L331) runs __str__
             each(Handed::Test) ─ report.line      (command L96)      Rust write + flush
           ◄── the gap: a Ctrl-C here waits for the next RESUME, in the next test
  └ back in the console script: sys.exit(_main())  CALL's check: traceback, exit 130
```

Links: [purba/src/lib.rs L29-L56](https://github.com/kalonji-tools/purba/blob/9051ab2/crates/purba/src/lib.rs#L29-L56), [command/src/lib.rs L94-L105](https://github.com/kalonji-tools/purba/blob/9051ab2/crates/command/src/lib.rs#L94-L105), [dispatch/src/lib.rs L70-L91](https://github.com/kalonji-tools/purba/blob/9051ab2/crates/dispatch/src/lib.rs#L70-L91), [execution/src/lib.rs L224-L233](https://github.com/kalonji-tools/purba/blob/9051ab2/crates/execution/src/lib.rs#L224-L233), [L299-L334](https://github.com/kalonji-tools/purba/blob/9051ab2/crates/execution/src/lib.rs#L299-L334), [assertion/src/lib.rs L167-L181](https://github.com/kalonji-tools/purba/blob/9051ab2/crates/assertion/src/lib.rs#L167-L181), [reporting/src/lib.rs L52-L104](https://github.com/kalonji-tools/purba/blob/9051ab2/crates/reporting/src/lib.rs#L52-L104).

**The lever.** The gap is microseconds wide. To widen it, `test_one` raises an exception whose message is 300 KB, larger than a pipe's buffer. purba's stdout is a FIFO whose reader waits 4 s before it reads. So purba blocks inside Rust's write of `test_one`'s line, after `test_one` returned and before `test_two` is called. At +1.5 s, `ps` shows purba in `anon_pipe_write`, and the log holds only `one start`. Then the signal goes to purba's process group.

| case | how the signal is sent | stdout | log of `test_two` | exit | tier |
|---|---|---|---|---|---|
| control, no signal (3.12.14, 3.14.4) | — | `test_one raised: Exception: xxx…`, `test_two passed`, `2 tests: 1 passed, 1 raised` | `two start` | 4 | [A] M2 |
| Ctrl-C in the gap (3.12.14, 3.14.4) | `kill -INT -- -<pid>` | `test_one raised: Exception: xxx…`, **`test_two raised: KeyboardInterrupt:`**, `2 tests: 2 raised` | **absent** | 4 | [A] M2 |
| Ctrl-C in the gap, real terminal (3.12.14, 3.14.4) | `^C` typed on a pty from `script(1)` | the same as the line above | **absent** | not captured | [A] M4 |
| Ctrl-C after the last test (3.12.14, 3.14.4) | `kill -INT -- -<pid>` | `test_one raised: Exception: xxx…`, `1 test: 1 raised`; stderr: a traceback at `sys.exit(_main())`, `KeyboardInterrupt` | — | **130**, not 4 | [A] M3 |
| syscall trace (3.14.4) | `kill -INT <pid>` | as in the gap case | `log.txt` opened once, by `test_one` | 4 | [A] M5 |

What the trace (M5) shows, in order: `write(1, "Exception: xxx…", 300011) = 62443`, then `--- SIGINT {si_code=SI_USER} ---`, then `write(1, "xxx…", 237568) = 237568`, then the line of `test_two`.
The signal cut the write short; Rust wrote the rest; no Python ran in between.
`test_two` never opened `log.txt`, so its first line never ran.
The source says where the raise happens: the eval breaker is set, and `test_two`'s `RESUME` is the first check that the main thread meets (S5). That location is read from source, not observed.

**What a falsifier would have shown, and did not:** `test_two passed` with `two start` in the log, a project error on `test_two` (an error that `getattr` raised before the call), or a `KeyboardInterrupt` charged to `test_one`.

## 4. The tester's code

Each run sends SIGINT to purba's process group (M6), on 3.12.14 and on 3.14.4, with the same result.

| the test | what reaches purba | what else happens | tier |
|---|---|---|---|
| installs `signal.signal(SIGINT, mine)` and does not restore it; Ctrl-C at +1 s (in `test_one`) and +4 s (in `test_two`) | nothing: `test_one passed`, `test_two passed`, exit 0 | `mine` ran twice, once in `test_one` and once in `test_two`; `test_two` saw `mine` as the handler. **A test's handler outlives its test** | [A] M6 |
| `try: time.sleep(3) except BaseException:`; Ctrl-C at +1 s | nothing: `test_one passed`, then `test_two` ran and passed, exit 0 | the test logged `caught KeyboardInterrupt` | [A] M6 |
| starts a thread and joins it; Ctrl-C at +1 s | `test_one raised: KeyboardInterrupt:`, raised in the main thread inside `join` | `test_one` ran on the main thread (`native id == pid`). The thread was not interrupted: it logged `worker done` while `test_two` ran | [A] M6 |
| the thread raises `KeyboardInterrupt` itself | nothing: `test_thread_raises_it passed` | `threading.excepthook` wrote the traceback to stderr | [A] M6 |
| `raise KeyboardInterrupt` | `raised: KeyboardInterrupt:` | — | [A] M6 |
| `signal.raise_signal(SIGINT)` | `raised: KeyboardInterrupt:` | — | [A] M6 |
| `os.kill(os.getpid(), SIGINT)` | `raised: KeyboardInterrupt:` | — | [A] M6 |
| a thread calls `_thread.interrupt_main()` | `raised: KeyboardInterrupt:` | — | [A] M6 |
| the exception's `__str__` sleeps 3 s; Ctrl-C at +1 s, while purba makes the line | `test_one raised: Slow: <exception str() failed>`; **the Ctrl-C is gone**; `test_two passed`; exit 4. Control without a signal: `Slow: slow` | PyO3's `Display` dropped the `KeyboardInterrupt` (section 2) | [A] M8 |
| returns a value whose `__del__` is slow | not reachable: selection refuses the test, `project error: the test returns a value, and purba does not read it` | — | [A] M8 |

## 5. A forked child: the planned zygote

[Each test runs in a process of its own](https://github.com/kalonji-tools/purba/blob/9051ab2/docs/decisions/each-test-runs-in-a-process-of-its-own.md#decision-outcome) decides "A zygote of purba's own loads the module once, and forks each test", and "On Windows, purba spawns each test".
It says nothing about process groups or signals.

| fact | source | tier |
|---|---|---|
| "A child created by fork(2) inherits its parent's session ID and process group ID." | [credentials.7 L84-L87](https://github.com/mkerrisk/man-pages/blob/ae6b221882ce71ba82fcdbe02419a225111502f0/man7/credentials.7#L84-L87) | [A] S13 |
| "When terminal keys that generate a signal (such as the interrupt key, normally control-C) are pressed, the signal is sent to the processes in the foreground job." | [credentials.7 L142-L145](https://github.com/mkerrisk/man-pages/blob/ae6b221882ce71ba82fcdbe02419a225111502f0/man7/credentials.7#L142-L145) | [A] S13 |
| CPython clears pending signal flags in a forked child: `PyOS_AfterFork_Child` calls `_PySignal_AfterFork`, "so that they aren't handled in both processes" | [posixmodule.c L755](https://github.com/python/cpython/blob/v3.14.10/Modules/posixmodule.c#L755), [signalmodule.c L2050-L2056](https://github.com/python/cpython/blob/v3.14.10/Modules/signalmodule.c#L2050-L2056) | [A] S1 |
| A child keeps the parent's Python handlers: CPython's handler table is process memory, and `fork` copies it | follows from S1 and `fork`; not probed with a custom handler | [C] |

A parent → zygote → test chain, made with `os.fork` in plain CPython 3.14.4 (M7):

| signal | parent | zygote | test child | tier |
|---|---|---|---|---|
| SIGINT to the parent's group (a terminal's Ctrl-C) | `KeyboardInterrupt` | `KeyboardInterrupt` | `KeyboardInterrupt` | [A] M7 |
| SIGINT to the parent's pid alone (`kill <pid>`) | `KeyboardInterrupt` | slept to the end | slept to the end, after the parent exited | [A] M7 |
| SIGINT to the group; the test child called `setpgid(0, 0)` | `KeyboardInterrupt` | `KeyboardInterrupt` | slept to the end | [A] M7 |

All three processes shared one session. A terminal's Ctrl-C therefore interrupts each test child **and** the zygote **and** purba at once, unless purba moves them out of the group.
A SIGTERM to purba's pid alone, as `kill` sends it, leaves the zygote and the test child running.
What a CI cancel and `timeout(1)` send, and to which processes, was not established here [C].

## 6. Windows

Nothing here ran on Windows. The facts come from Microsoft's documentation and CPython's source.

| fact | source | tier |
|---|---|---|
| Ctrl-C and Ctrl-Break are "passed to all console processes that are attached to the console". "The system creates a new thread in each client process to handle the event." | [ctrl-c-and-ctrl-break-signals.md L20](https://github.com/MicrosoftDocs/Console-Docs/blob/8cf64f2ce92961421c4fb95f08ce3ee932893028/docs/ctrl-c-and-ctrl-break-signals.md#L20) | [A] W1 |
| Handlers run "on a last-registered, first-called basis until one of the handlers returns TRUE"; the default handler "calls the ExitProcess function" | [setconsolectrlhandler.md L70](https://github.com/MicrosoftDocs/Console-Docs/blob/8cf64f2ce92961421c4fb95f08ce3ee932893028/docs/setconsolectrlhandler.md#L70), [handlerroutine.md L66](https://github.com/MicrosoftDocs/Console-Docs/blob/8cf64f2ce92961421c4fb95f08ce3ee932893028/docs/handlerroutine.md#L66) | [A] W2 |
| `CTRL_C_EVENT` "cannot be limited to a specific process group"; a group is made with `CREATE_NEW_PROCESS_GROUP` | [generateconsolectrlevent.md L54-L60](https://github.com/MicrosoftDocs/Console-Docs/blob/8cf64f2ce92961421c4fb95f08ce3ee932893028/docs/generateconsolectrlevent.md#L54-L60) | [A] W3 |
| CRT: "When a CTRL+C interrupt occurs, Win32 operating systems generate a new thread to specifically handle that interrupt." "By default, signal terminates the calling program with exit code 3." A handler resets to `SIG_DFL` before it runs | [cpp-docs signal.md L54-L67](https://github.com/MicrosoftDocs/cpp-docs/blob/f29e25fb8f05bb38cf0246476b5c1fc92bf33876/docs/c-runtime-library/reference/signal.md#L54-L67) | [A] W4 |
| CPython re-installs its C handler on each signal where it has no `sigaction`, and on SIGINT it sets `sigint_event`. `trip_signal`'s comment: "especially on windows, where trip_signal may run in a new thread" | [signalmodule.c L286-L288](https://github.com/python/cpython/blob/v3.14.10/Modules/signalmodule.c#L286-L288), [L349-L379](https://github.com/python/cpython/blob/v3.14.10/Modules/signalmodule.c#L349-L379) | [A] S1 |
| `time.sleep` on the main thread waits on the timer and `sigint_event`, then calls `PyErr_CheckSignals` | [timemodule.c L2345-L2366](https://github.com/python/cpython/blob/v3.14.10/Modules/timemodule.c#L2345-L2366) | [A] S6 |
| `signal.signal` accepts only SIGABRT, SIGFPE, SIGILL, SIGINT, SIGSEGV, SIGTERM and SIGBREAK; `CTRL_C_EVENT` and `CTRL_BREAK_EVENT` "can only be used with `os.kill`" | [signal.rst L625-L627](https://github.com/python/cpython/blob/v3.14.10/Doc/library/signal.rst?plain=1#L625-L627), [L292-L307](https://github.com/python/cpython/blob/v3.14.10/Doc/library/signal.rst?plain=1#L292-L307) | [A] S4 |
| CPython installs a Python handler for SIGINT only, so Ctrl-Break (SIGBREAK) meets the CRT default | [signalmodule.c L1602-L1614](https://github.com/python/cpython/blob/v3.14.10/Modules/signalmodule.c#L1602-L1614), W4 | [A] by reading; [C] what the tester sees |
| An extension can call `SetConsoleCtrlHandler` itself; its handler runs before the CRT's, and on a thread of its own | W2 | [A] docs; [C] not tried with PyO3 |
| How the spawned test of each Windows test receives Ctrl-C, and what purba's exit code is | — | [C] |

## 7. Where purba can tell a signal from a raise

Today, nowhere. Section 4 shows five ways to get the same line, `raised: KeyboardInterrupt:`. Section 3 shows a sixth: a Ctrl-C in purba's gap, charged to the next test.

What each point could tell, if purba had the means named in its row. These rows are the points that exist in the code at 9051ab2; they are not a design.

| point in the run | what is knowable there | what purba needs | what it cannot tell | tier |
|---|---|---|---|---|
| **return of `test.call0()`** ([execution L315](https://github.com/kalonji-tools/purba/blob/9051ab2/crates/execution/src/lib.rs#L315)) with `Err(KeyboardInterrupt)` | whether a signal was handled during the call | a SIGINT handler of purba's own that counts each call, read before and after the call | `os.kill(self)`, `raise_signal` and `interrupt_main` count as signals; a test that replaced the handler hides them (M6) | reasoning on M6 |
| **return of `test.call0()`** with `Ok` | that the test swallowed a Ctrl-C (`except BaseException`, own handler) | the same count | — | reasoning on M6 |
| **between two tests** (before [execution L304](https://github.com/kalonji-tools/purba/blob/9051ab2/crates/execution/src/lib.rs#L304)) | that a signal is pending and belongs to no test | `py.check_signals()` | a signal that arrives after the check and before `RESUME` still reaches the test | [A] S10; the residue is reasoning |
| **after `signal.getsignal(SIGINT)` once a test returns** | that the test replaced purba's handler | a compare against purba's handler | — | [A] M6 shows a handler outlives its test |
| **inside the C-level handler, with `SA_SIGINFO`** | who sent it: `SI_KERNEL` with `si_pid=0` for a terminal's `^C`; `SI_USER` with the sender's pid for `kill`; `SI_USER` with purba's own pid for a test's `os.kill` or `raise_signal` | a `sigaction` below CPython's, which must chain to CPython's handler | `interrupt_main` reaches no OS handler; macOS not probed | [A] M10 on Linux; [C] macOS |
| **when purba makes a line** ([execution L329-L331](https://github.com/kalonji-tools/purba/blob/9051ab2/crates/execution/src/lib.rs#L329-L331)) | nothing today: PyO3 drops the error of `__str__` | a `str()` call that keeps its error | — | [A] M8 |
| **in the zygote's parent** | whether a child died of a signal (`waitpid` status) or ended with a `KeyboardInterrupt` it reported | the zygote design | — | [C] |

## What this research did not establish

- How often the gap catches a Ctrl-C without the lever. The gap is a few Rust calls and one flushed write [C].
- Free-threaded CPython 3.14t. mise's 3.14.7 is a free-threaded build, and the abi3 wheel does not install on it [C].
- macOS. Process groups follow POSIX there too, but no probe ran [C].
- Windows in full: section 6 is documents and source only [C for behaviour].
- What a CI cancel sends, and to which processes [C].

## Raw evidence

Scripts and outputs live in the session scratchpad `r2/`. Each probe script is named below; its output is pasted.

### M0. Code reads and wheel provenance

```text
$ git diff --stat 79288dc 9051ab2 -- crates Cargo.lock Cargo.toml pyproject.toml
 crates/execution/tests/run.rs | 46 +++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 46 insertions(+)
$ grep -rn -E "check_signals|signal|KeyboardInterrupt|allow_threads|detach" crates/*/src
crates/parsing/src/lib.rs:59:        let detached = Detached::default();
crates/parsing/src/lib.rs:60:        detached
crates/parsing/src/lib.rs:64:        while detached.free_last(&detached.exprs, walk_expr)
crates/parsing/src/lib.rs:65:            || detached.free_last(&detached.stmts, walk_stmt)
crates/parsing/src/lib.rs:66:            || detached.free_last(&detached.patterns, walk_pattern)
crates/parsing/src/lib.rs:67:            || detached.free_last(&detached.elements, walk_interpolated_string_element)
$ grep -n -A2 'name = "pyo3"' Cargo.lock
name = "pyo3"
version = "0.29.2"
```

The wheel: `purba-0.0.0-cp312-abi3-manylinux_2_34_x86_64.whl`, sha256 `2632539707eff5083c8b5e8cf920b349b8f4e5bccc16e8baf6ebb7846e82dbd5`, built from `79288dc`.
Installed in a CPython 3.12.14 venv (mise) and a CPython 3.14.4 venv (uv). The console script `bin/purba` ends in `sys.exit(_main())`.

### M1. Default handlers

```text
$ python -c "import signal,sys; print(sys.version.split()[0], 'SIGINT:', signal.getsignal(signal.SIGINT), '| SIGTERM:', signal.getsignal(signal.SIGTERM), '| SIGPIPE:', signal.getsignal(signal.SIGPIPE))"
3.12.14 SIGINT: <built-in function default_int_handler> | SIGTERM: 0 | SIGPIPE: 1
3.14.4 SIGINT: <built-in function default_int_handler> | SIGTERM: 0 | SIGPIPE: 1
```

### M2, M3. The gap probe (`gap.sh <venv> control|gap|gap-last`)

```bash
set -um
# test_gap.py:
#   def test_one():  LOG.open("a").write("one start\n"); raise Exception("x" * 300000)
#   def test_two():  LOG.open("a").write("two start\n"); assert 1 == 1
#   (gap-last: test_one only)
mkfifo fifo
(exec <fifo; sleep 4; cat >out) &            # reader: its own job, its own group
reader=$!
"$VENV/bin/purba" test_gap.py >fifo 2>err &
pid=$!
sleep 1.5
ps -o pid,pgid,sid,stat,wchan:20,comm -p "$pid","$reader"
[ "$CASE" != control ] && kill -INT -- -"$pid"
wait "$pid"; code=$?
```

```text
--- at +1.5s: purba pid=1993909
    PID    PGID     SID STAT WCHAN                COMMAND
1993909 1993909 1993725 S    anon_pipe_write      purba
1993908 1993908 1993725 S    do_wait              bash
log so far: one start
sending: kill -INT -- -1993909
### gap on Python 3.12.14: exit=4
--- stdout (each line cut to 100 chars)
test_gap.py::test_one raised: Exception: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
test_gap.py::test_two raised: KeyboardInterrupt:
2 tests: 2 raised
--- stderr
--- log
one start

### control on Python 3.12.14: exit=4
test_gap.py::test_one raised: Exception: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
test_gap.py::test_two passed
2 tests: 1 passed, 1 raised
--- log
one start
two start

### gap-last on Python 3.12.14: exit=130
--- stdout (each line cut to 100 chars)
test_gap.py::test_one raised: Exception: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
1 test: 1 raised
--- stderr
Traceback (most recent call last):
  File ".../int/venv/bin/purba", line 8, in <module>
    sys.exit(_main())
             ^^^^^^^
KeyboardInterrupt
--- log
one start
```

The 3.14.4 runs printed the same lines, with `python3` as the command name, the same exits (4, 4, 130), and the traceback marker `~~~~~^^`.

### M4. The gap probe with a real terminal's `^C` (`gap-pty.sh <venv>`)

```bash
(sleep 1.5; printf '\003'; sleep 4) |
  script -qec "$VENV/bin/purba test_gap.py >fifo 2>err; echo \"purba exit=\$?\"" /dev/null
```

```text
log at +1.5s: one start
^C--- stdout (cut to 100)
test_gap.py::test_one raised: Exception: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
test_gap.py::test_two raised: KeyboardInterrupt:
2 tests: 2 raised
--- stderr
--- log
one start
```

The same on 3.12.14 and 3.14.4. The `echo` of the exit did not print: the `sh` that `script` started is in the same foreground group, and the `^C` ended it too.

### M5. The gap under strace (`gap-strace.sh`, 3.14.4)

```bash
strace -o trace -e trace=write,openat -e signal=SIGINT -s 20 "$VENV/bin/purba" test_gap.py >fifo 2>err &
kill -INT <pid of purba under strace>
grep -n -E 'SIGINT|EINTR|resumed|write\(1|log.txt' trace
```

```text
exit=4
129:openat(AT_FDCWD, "log.txt", O_WRONLY|O_CREAT|O_APPEND|O_CLOEXEC, 0666) = 3
131:write(1, "test_gap.py::test_on"..., 30) = 30
132:write(1, "Exception: xxxxxxxxx"..., 300011) = 62443
133:--- SIGINT {si_signo=SIGINT, si_code=SI_USER, si_pid=2006820, si_uid=1000} ---
134:write(1, "xxxxxxxxxxxxxxxxxxxx"..., 237568) = 237568
135:write(1, "\n", 1)                       = 1
136:write(1, "test_gap.py::test_tw"..., 49) = 49
137:write(1, "2 tests: 2 raised\n", 18)     = 18
```

### M6. The tester's code (`tester.sh <venv>`)

Each `run` starts purba in the background with `set -m`, and sends `kill -INT -- -<pid>` at the delays shown.

```text
Python 3.12.14
### a test installs its own handler (SIGINT to the group at +1 3s) exit=0
test_handler.py::test_one passed
test_handler.py::test_two passed
2 tests: 2 passed
--- log
handler of test_one: signal 2 in test_one
handler of test_one: signal 2 in test_two
test_two sees handler mine

### a test catches BaseException (SIGINT to the group at +1s) exit=0
test_catch.py::test_one passed
test_catch.py::test_two passed
2 tests: 2 passed
--- log
test_one caught KeyboardInterrupt
two start

### a test joins a thread (SIGINT to the group at +1s) exit=4
test_thread.py::test_one raised: KeyboardInterrupt:
test_thread.py::test_two passed
2 tests: 1 passed, 1 raised
--- log
test_one on main thread? True, native id == pid? True
worker starts, main thread? False
two start
worker done

### a test makes the interrupt itself (no signal sent) exit=4
test_self.py::test_raise raised: KeyboardInterrupt:
test_self.py::test_raise_signal raised: KeyboardInterrupt:
test_self.py::test_kill_self raised: KeyboardInterrupt:
test_self.py::test_interrupt_main_from_a_thread raised: KeyboardInterrupt:
test_self.py::test_thread_raises_it passed
test_self.py::test_last passed
6 tests: 2 passed, 4 raised
--- stderr
Exception in thread Thread-2 (work):
Traceback (most recent call last):
  ...
  File "test_self.py", line 19, in work
    raise KeyboardInterrupt
KeyboardInterrupt
```

The 3.14.4 run printed the same stdout, logs and exits.

### M7. A fork chain (`fork.sh`, plain CPython 3.14.4)

```text
### SIGINT to the parent's group (a terminal's Ctrl-C)
parent: KeyboardInterrupt
parent: pid=1997073 pgid=1997073 sid=1997060
test: KeyboardInterrupt
test: pid=1997076 pgid=1997073 sid=1997060
zygote: KeyboardInterrupt
zygote: pid=1997075 pgid=1997073 sid=1997060

### SIGINT to the parent's pid alone (kill <pid>)
parent: KeyboardInterrupt
parent: pid=1997211 pgid=1997211 sid=1997060
test: pid=1997214 pgid=1997211 sid=1997060
test: slept to the end
zygote: pid=1997213 pgid=1997211 sid=1997060
zygote: slept to the end

### SIGINT to the group, the test child in a group of its own
parent: KeyboardInterrupt
parent: pid=1997427 pgid=1997427 sid=1997060
test: pid=1997430 pgid=1997430 sid=1997060
test: slept to the end
zygote: KeyboardInterrupt
zygote: pid=1997429 pgid=1997427 sid=1997060
```

### M8. The tester's code that purba runs outside the call (`outside.sh <venv>`)

```text
### test_str.py (SIGINT to the group at +1s) exit=4
--- stdout
test_str.py::test_one raised: Slow: <exception str() failed>
test_str.py::test_two passed
2 tests: 1 passed, 1 raised
--- log
__str__ starts
two start

control, no signal:
test_str.py::test_one raised: Slow: slow
test_str.py::test_two passed
2 tests: 1 passed, 1 raised
exit=4

### test_del.py (SIGINT to the group at +1s) exit=4
test_del.py::test_one project error: the test returns a value, and purba does not read it
test_del.py::test_two passed
2 tests: 1 passed, 1 blocked; 1 project error
```

The same on 3.12.14 and 3.14.4. `test_del.py` called no test with a slow `__del__`, so purba had exited before +1 s, and the shell printed `kill: (-2020164) - No such process`.

### M9. An inherited `SIG_IGN` (`inherit.sh <venv> without-m|with-m`, 3.14.4)

```text
SigIgn:	0000000001001006
### without-m: SIGINT to purba's pid at +1s, exit=0
test_inherit.py::test_one passed
1 test: 1 passed
SIGINT handler in the test: 1

SigIgn:	0000000001001000
### with-m: SIGINT to purba's pid at +1s, exit=4
test_inherit.py::test_one raised: KeyboardInterrupt:
1 test: 1 raised
SIGINT handler in the test: <built-in function default_int_handler>
```

Bit 1 of `SigIgn` is SIGINT. POSIX 2.12: "If job control is disabled … when the shell executes an asynchronous AND-OR list, the commands in the list shall inherit from the shell a signal action of ignored (SIG_IGN) for the SIGINT and SIGQUIT signals."

### M10. Who sent SIGINT: `siginfo` (`siginfo.sh`, 3.14.4)

The script blocks SIGINT and reads it with `signal.sigwaitinfo`.

```text
self-kill: si_code=0 (SI_USER) si_pid=2000500 (self=2000500, parent=2000496)
raise-signal: si_code=0 (SI_USER) si_pid=2000504 (self=2000504, parent=2000496)
pid 2000505 waits
shell pid 2000496 sends kill -INT -- -2000505
wait: si_code=0 (SI_USER) si_pid=2000496 (self=2000505, parent=2000496)
--- through a pty, ^C typed:
pid 2000559 waits
^Cwait: si_code=128 (SI_KERNEL) si_pid=0 (self=2000559, parent=2000557)
```

`sigaction(2)`: "SI_USER kill(2)", "SI_KERNEL Sent by the kernel" ([sigaction.2 L648-L656](https://github.com/mkerrisk/man-pages/blob/ae6b221882ce71ba82fcdbe02419a225111502f0/man2/sigaction.2#L648-L656)).

### Sources fetched

| id | what | command |
|---|---|---|
| S1–S9 | CPython at `v3.14.10` (the newest 3.14 tag on 2026-11-04): `Modules/signalmodule.c`, `Python/pylifecycle.c`, `Python/ceval_gil.c`, `Include/internal/pycore_pystate.h`, `Python/bytecodes.c`, `Modules/timemodule.c`, `Modules/main.c`, `Python/pythonrun.c`, `Modules/posixmodule.c`, `Doc/library/signal.rst`, `Doc/library/_thread.rst`, `Doc/c-api/exceptions.rst` | `curl -sfL https://raw.githubusercontent.com/python/cpython/v3.14.10/<path>`; tag from `git ls-remote --tags https://github.com/python/cpython 'v3.14.*'` |
| S10–S11 | PyO3 0.29.2 `src/marker.rs`, `src/err/mod.rs`, `pyo3-ffi/src/pyerrors.rs` | read in `~/.cargo/registry/src/*/pyo3-0.29.2`; `curl … PyO3/pyo3/v0.29.2/<path> \| diff -q - <registry file>` printed no difference |
| S12 | Rust `library/core/src/io/write.rs` at `1.99.1` | `curl -sfL https://raw.githubusercontent.com/rust-lang/rust/1.99.1/library/core/src/io/write.rs` |
| S13 | man-pages at `ae6b2218`: `man7/credentials.7`, `man2/sigaction.2` | `curl -sfL https://raw.githubusercontent.com/mkerrisk/man-pages/ae6b2218…/<path>` |
| W1–W3 | Console-Docs at `8cf64f2c`: `ctrl-c-and-ctrl-break-signals.md`, `setconsolectrlhandler.md`, `handlerroutine.md`, `generateconsolectrlevent.md` | `curl -sfL https://raw.githubusercontent.com/MicrosoftDocs/Console-Docs/main/docs/<file>`; commit from `git ls-remote` |
| W4 | cpp-docs at `f29e25fb`: `docs/c-runtime-library/reference/signal.md` | `curl -sfL https://raw.githubusercontent.com/MicrosoftDocs/cpp-docs/main/docs/c-runtime-library/reference/signal.md` |
| POSIX | Shell Command Language, 2.12 Signals and Error Handling | `curl -sfL https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html` |
