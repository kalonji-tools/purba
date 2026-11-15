"""#1010 Q3: a stage of another file, run with this file alone. The runner has no verdict for the
stage, so the replay runs its checks: a stage that fails blocks the test and names the check. Run it
alone, as `run.py test_use_far.py`."""

from __future__ import annotations

from cases import use
from test_use_stage import Filled, ann_books, wrong


def test_far_checkout(filled: Filled = use(ann_books)) -> None:
    """Passes: the stage's checks hold on the replay."""
    order = filled.shop.checkout(filled.user)
    assert order.total == 2400


def test_far_after_wrong(filled: Filled = use(wrong)) -> None:
    """Blocked: the replay of `wrong` fails its check `len(...) == lines`, and the line says so."""
    order = filled.shop.checkout(filled.user)
    assert order.total == 1200
