# Of the exceptions CPython ignores, only a thread's is a project error

## Context and Problem Statement

[A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) makes a thread's exception a project error.
CPython hands the exception of a thread that `_thread` starts to `sys.unraisablehook`.
On `main` at `03d2170`, purba reads only `threading.excepthook`.
No record says which exceptions on `sys.unraisablehook` are a project error.

| fact | source |
|---|---|
| on `main` at `03d2170`, when a thread that `_thread` starts raises in a test, purba prints `passed` for the test, and the run exits 0 | [An exception in a thread that _thread started gives no error, and the run exits 0](https://github.com/kalonji-tools/purba/issues/816) |
| `sys.unraisablehook` also takes the exception of a `__del__`, a weakref callback, a generator that closes, and an `atexit` function | [the spec of that issue](https://github.com/kalonji-tools/purba/issues/816#issuecomment-6739544129) |
| the message of a call to `sys.unraisablehook` from a thread that `_thread` starts begins with `Exception ignored in thread started by` | [CPython 3.12 `_threadmodule.c`](https://github.com/python/cpython/blob/v3.12.0/Modules/_threadmodule.c#L1088) and [`errors.c`](https://github.com/python/cpython/blob/v3.12.0/Python/errors.c#L1646), [CPython 3.14 `_threadmodule.c`](https://github.com/python/cpython/blob/v3.14.0/Modules/_threadmodule.c#L365-L366) |
| from CPython 3.13 on, that call carries no object | [CPython 3.13 `_threadmodule.c`](https://github.com/python/cpython/blob/v3.13.0/Modules/_threadmodule.c#L343-L344) and [`errors.c`](https://github.com/python/cpython/blob/v3.13.0/Python/errors.c#L1680-L1688) |
| CPython drops a `SystemExit`, or a subclass of it, that such a thread raises, and calls no hook | [CPython 3.12 `_threadmodule.c`](https://github.com/python/cpython/blob/v3.12.0/Modules/_threadmodule.c#L1083-L1086), [CPython 3.14 `_threadmodule.c`](https://github.com/python/cpython/blob/v3.14.0/Modules/_threadmodule.c#L359-L363) |

## Considered Options

- **No exception on `sys.unraisablehook` is a project error.** One exception is then a project error in a `threading.Thread`, and no error in a thread that `_thread` starts. The run can then exit 0.

## Decision Outcome

**Reach:** `crates/**` `tests/**`

**Area:** errors

**An exception that CPython ignores is a project error only when a thread that `_thread` starts raised it.**

- **purba tells a call to `sys.unraisablehook` from such a thread by CPython's message.** A call whose message begins with `Exception ignored in thread started by` comes from such a thread. purba cannot tell such a thread by the object of the call, because a newer CPython gives no object.
- **purba keeps the exception, and CPython's own hook still prints the call.** [An interrupt puts back no more than a normal end does](an-interrupt-puts-back-no-more-than-a-normal-end-does.md#considered-options) rejects a hook that hides the error of a test's `atexit` function.
- **A call from such a thread charges the test or the load that runs when the thread raises.** The call carries no thread, so purba cannot find the test or the load that left the thread alive.
- **A `SystemExit`, or a subclass of it, from such a thread gives no error.** CPython drops it and calls no hook.

**Downside:**

- **A CPython that changes the message hides the thread's exception again.** The run can then exit 0.
- **The exception of a `__del__`, a weakref callback or a generator in a test gives no error.** CPython prints it, and the run can exit 0.
- **A tester who replaces `sys.unraisablehook` hides from purba the exception of a thread that `_thread` starts.** The run can then exit 0.

## Confirmation

`mise run test:wheel` checks that purba prints a project error for the exception of a thread that `_thread` starts.
It checks that stderr still holds CPython's message.
It checks that purba prints the test as `raised`, and no project error, when such a thread calls `os._exit`.
It checks that a `__del__` that raises in a test gives no error.
The `wheel` job of `ci.yml` runs these checks on each Python of its matrix.

`mise run test:rust` checks that such an exception charges the test that runs when the thread raises, not the test that started it.
No command checks that a `SystemExit` from such a thread gives no error.
