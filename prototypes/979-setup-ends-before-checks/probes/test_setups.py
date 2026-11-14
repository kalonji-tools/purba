"""Probes of #979: each test tries to falsify one answer. The expected screen of each is in its docstring."""

import contextlib
import io
import os
import signal
import sys
import threading
import warnings

from cases import case, raises

STATE = {"clock": "real"}


@contextlib.contextmanager
def frozen(at: str):
    """A stand-in for freeze_time: sets a module state, and puts it back on the way out."""
    STATE["clock"] = at
    try:
        yield at
    finally:
        STATE["clock"] = "real"


class BadUndo:
    def __enter__(self) -> None:
        return None

    def __exit__(self, *_: object) -> None:
        raise OSError("the undo broke")


class BadEntry:
    def __enter__(self) -> None:
        raise OSError("the setup broke")

    def __exit__(self, *_: object) -> None:
        return None


class Swallow:
    """Swallows any raise, as `logger.catch()` does."""

    def __enter__(self) -> None:
        return None

    def __exit__(self, kind: object, *_: object) -> bool:
        return kind is not None


def boom() -> int:
    raise KeyError("k")


# ── Q1, Q5: a setup ends before the checks; checks inside it, after it, or both ──


def test_checks_after_the_undo():
    """✓ both: the clock was frozen inside, and is back after."""
    with frozen("1999-12-12"):
        inside = STATE["clock"]
    after = STATE["clock"]
    assert inside == "1999-12-12"
    assert after == "real"


checks_after_the_undo = case(test_checks_after_the_undo)


def test_checks_inside_the_setup():
    """✓ both: each check reads under the setup; the first in a fork, the last in place."""
    with frozen("2001-02-03"):
        seen = STATE["clock"]
        assert seen == "2001-02-03"
        assert STATE["clock"] == "2001-02-03"


checks_inside_the_setup = case(test_checks_inside_the_setup)


def test_checks_inside_and_after():
    """✓ all three: two under the setup, one after its undo."""
    with frozen("2001-02-03"):
        seen = STATE["clock"]
        assert seen == "2001-02-03"
        assert STATE["clock"] == "2001-02-03"
    assert STATE["clock"] == "real"


checks_inside_and_after = case(test_checks_inside_and_after)


# ── Q9: nested and multi-item setups, and an expecter inside a setup ──


def test_nested_and_multi_item():
    """✓: two setups on one line, a third inside them."""
    with frozen("a"), contextlib.nullcontext():
        with contextlib.nullcontext():
            seen = STATE["clock"]
            assert seen == "a"
    assert STATE["clock"] == "real"


nested_and_multi_item = case(test_nested_and_multi_item)


def test_raises_inside_a_setup():
    """✓ raises KeyError, then the check after the undo."""
    with frozen("a"):
        with raises(KeyError):
            boom()
    assert STATE["clock"] == "real"


raises_inside_a_setup = case(test_raises_inside_a_setup)


# ── Q4: a setup that swallows a raise ──


def test_swallow_at_the_last_line():
    """✓: the raise is the body's last statement, so nothing was skipped (loguru's `with logger.catch(): raise …`)."""
    with Swallow():
        boom()
    assert STATE["clock"] == "real"


swallow_at_the_last_line = case(test_swallow_at_the_last_line)


def test_swallow_skips_a_line():
    """✗ setup: line of boom() raised, the next line never ran."""
    got = []
    with contextlib.suppress(KeyError):
        boom()
        got.append(1)
    assert got == []


swallow_skips_a_line = case(test_swallow_skips_a_line)


def test_swallow_skips_a_check():
    """✗ setup: the check inside never ran."""
    with Swallow():
        value = boom()
        assert value == 1


swallow_skips_a_check = case(test_swallow_skips_a_check)


# ── Q8: an undo or a setup that raises ──


def test_undo_raises():
    """✗ undo: names BadUndo and its line; the check inside ran, the check after is blocked."""
    with BadUndo():
        seen = STATE["clock"]
        assert seen == "real"
    assert STATE["clock"] == "real"


undo_raises = case(test_undo_raises)


def test_setup_raises():
    """✗ setup: names BadEntry and its line."""
    with BadEntry():
        seen = STATE["clock"]
    assert seen == "real"


setup_raises = case(test_setup_raises)


# ── Q2: the act replaces a hook the runner sees faults through ──


@contextlib.contextmanager
def quiet_threads():
    old = threading.excepthook
    threading.excepthook = lambda args: None
    try:
        yield
    finally:
        threading.excepthook = old


def fail_in_a_thread() -> None:
    raise ValueError("in the thread")


def test_thread_hook_swapped_in_a_window():
    """✗ hook threading.excepthook, at quiet_threads; the thread's error is not seen (the hook was swapped)."""
    with quiet_threads():
        worker = threading.Thread(target=fail_in_a_thread)
        worker.start()
        worker.join()
    assert worker.is_alive() is False


thread_hook_swapped_in_a_window = case(test_thread_hook_swapped_in_a_window)


def test_warnings_muted_in_a_window():
    """✗ hook warnings: catch_warnings swaps the filters."""
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        warnings.warn("old api", DeprecationWarning)
        got = 1
    assert got == 1


warnings_muted_in_a_window = case(test_warnings_muted_in_a_window)


def test_warnings_muted_for_good():
    """✗ hook warnings.filters (simplefilter changes the list in place)."""
    warnings.simplefilter("ignore")
    warnings.warn("old api", DeprecationWarning)
    got = 1
    assert got == 1


warnings_muted_for_good = case(test_warnings_muted_for_good)


def test_unraisable_swapped():
    """✗ hook sys.unraisablehook."""
    sys.unraisablehook = lambda u: None
    got = 1
    assert got == 1


unraisable_swapped = case(test_unraisable_swapped)


def test_alarm_cancelled():
    """✗ hook signal.alarm: the runner's time limit."""
    signal.alarm(0)
    got = 1
    assert got == 1


alarm_cancelled = case(test_alarm_cancelled)


# ── Q6: stdout and stderr are free ──


def test_stdout_redirected():
    """✓: the test reads what it printed; the runner's screen is unaffected."""
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer):
        print("hello")
    assert buffer.getvalue() == "hello\n"


stdout_redirected = case(test_stdout_redirected)


def test_stderr_none():
    """✓: loguru's test_no_sys_stderr sets sys.stderr to None; the runner still reports."""
    saved = sys.stderr
    sys.stderr = None
    got = os.getpid() > 0
    sys.stderr = saved
    assert got is True


stderr_none = case(test_stderr_none)


# ── refusals that stay ──


def test_assert_inside_the_act():
    """! refused: an assert, then more act."""
    with frozen("a"):
        assert STATE["clock"] == "a"
        seen = STATE["clock"]
    assert seen == "a"


assert_inside_the_act = case(test_assert_inside_the_act)


def test_setup_mixed_with_raises():
    """! refused: raises takes a `with` of its own."""
    with frozen("a"), raises(KeyError):
        boom()


setup_mixed_with_raises = case(test_setup_mixed_with_raises)
