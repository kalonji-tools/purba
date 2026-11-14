"""Probes of #979: the guards still see what they guard, and a module that replaces sys.stdout cannot mute the runner.

Prototype 5's port m2b replaced sys.stdout at module level, and the runner printed 0 bytes. Expected screen below.
"""

import io
import sys
import threading
import warnings

from cases import case, warns

sys.stdout = io.StringIO()  # as m2b's module-level capture did


def noisy() -> int:
    warnings.warn("old api", DeprecationWarning)
    return 1


def fail_in_a_thread() -> None:
    raise ValueError("in the thread")


def test_a_warning_is_seen():
    """✗ warning: DeprecationWarning old api; ✓ got == 1."""
    got = noisy()
    assert got == 1


a_warning_is_seen = case(test_a_warning_is_seen)


def test_an_expected_warning_passes():
    """✓ warns, ✓ got == 1: the form's own `warns` changes the filters, and the guard lets it."""
    with warns(DeprecationWarning):
        got = noisy()
    assert got == 1


an_expected_warning_passes = case(test_an_expected_warning_passes)


def test_a_thread_error_is_seen():
    """✗ lost error: in a thread, ValueError."""
    worker = threading.Thread(target=fail_in_a_thread)
    worker.start()
    worker.join()
    assert worker.is_alive() is False


a_thread_error_is_seen = case(test_a_thread_error_is_seen)
