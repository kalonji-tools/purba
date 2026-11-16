"""#1015, change 8: a case line shows its wall time when the case took 1 s or more, and the summary
line names the slowest case."""

from __future__ import annotations

import time

from shop import Shop


def test_slow() -> None:
    """PASS, in about 1.2 s. Says "✓ test_slow › shop.users == ['ann']   ['ann']  1.2 s" """
    time.sleep(1.2)
    shop = Shop()
    shop.sign_up("ann")
    assert shop.users == ["ann"]


def test_slow_and_failing() -> None:
    """FAIL, in about 1.0 s; the time sits on the case's first line. Says "  1.0 s" """
    time.sleep(1.0)
    shop = Shop()
    shop.sign_up("ann")
    assert shop.users == ["bob"]


def test_quick() -> None:
    """PASS, quickly: no time is shown."""
    assert Shop().users == []


def test_slow_and_refused(name: str = "") -> None:
    """REFUSED: the `or` goes one way; the time shows anyway. Says " s" """
    time.sleep(1.0)
    shown = name or "anonymous"
    assert shown == "anonymous"
