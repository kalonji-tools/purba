"""Probes of #979: where `FORM_GUARD=line` and `FORM_GUARD=swap` differ."""

import threading
import warnings

from cases import case
from probe_lib import quietly


def noisy() -> int:
    warnings.warn("old api", DeprecationWarning)
    return 1


def fail_in_a_thread() -> None:
    raise ValueError("in the thread")


def test_library_mutes_inside_one_call():
    """swap: ✗ hook at probe_lib.py. line: ✓ — the warning of `noisy` is lost (the hole of line mode)."""
    got = quietly(noisy)
    assert got == 1


library_mutes_inside_one_call = case(test_library_mutes_inside_one_call)


def test_swap_undone_by_a_later_line():
    """swap: ✗ hook. line: ✗ hook — the step after the swap's line sees it."""
    old = threading.excepthook
    threading.excepthook = lambda args: None
    worker = threading.Thread(target=fail_in_a_thread)
    worker.start()
    worker.join()
    threading.excepthook = old
    assert worker.is_alive() is False


swap_undone_by_a_later_line = case(test_swap_undone_by_a_later_line)
