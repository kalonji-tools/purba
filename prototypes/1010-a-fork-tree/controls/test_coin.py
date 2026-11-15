"""INJECTED CONTROL, not corpus evidence (#1010): a stage whose act reads `random`.

Its own case passes half the time. Under replay, a later case runs the toss again, so it is
blocked as "diverged on replay" whenever the second toss differs. The fork tree tosses once,
so it never sees that drift.
"""

from __future__ import annotations

from cases import case
from coin import toss


def test_toss() -> str:
    side = toss()
    assert side == "heads"
    return side


tossed = case(test_toss)


def test_call(side: str) -> None:
    called = side.upper()
    assert called == "HEADS"


called = case(test_call, side=tossed.product)
