"""#1015, change 3: `@skip(reason)` and `@skip_if(condition, reason)`. The reason is a string literal
that is not empty. A skipped case never runs, and a later stage that takes its product is
blocked."""

from __future__ import annotations

import sys

from cases import before, case, skip, skip_if, use
from registry import names, register, reset
from shop import Shop


@skip("the shop has no refunds of a refund yet")
def test_skipped(name: str) -> Shop:
    """SKIPPED. Says "○ ann  skipped: the shop has no refunds of a refund yet" """
    shop = Shop()
    shop.sign_up(name)
    assert shop.users == [name]
    return shop


ann = case(test_skipped, name="ann")
bob = case(test_skipped, name="bob")


def test_after_a_skipped_stage(shop: Shop = use(ann)) -> None:
    """BLOCKED. Says "it takes the product of ann (test_1015_skip.py:23), which was skipped" """
    shop.sign_up("cy")
    assert shop.users == ["ann", "cy"]


@skip_if(sys.platform != "nowhere", "the condition holds on every platform")
@before(reset)
def test_skip_if_holds() -> None:
    """SKIPPED. Says "skipped: the condition holds on every platform" """
    register("ann")
    assert names() == ["ann"]


@skip_if(sys.platform == "nowhere", "the condition holds on no platform")
def test_skip_if_does_not_hold() -> None:
    """FAIL: the condition is false, so the case runs, and "builtin" is in the registry."""
    register("ann")
    assert names() == ["ann"]


@skip_if(False, "never")
def test_skip_if_passes() -> None:
    """PASS: the condition is false, so the case runs."""
    shop = Shop()
    shop.sign_up("ann")
    assert shop.users == ["ann"]


REASON = "a name"


@skip(REASON)
def test_reason_by_name() -> None:
    """REFUSED before the run: the reason is a name, not a literal. Says "`@skip(…)` takes its
    reason as a string literal that is not empty, and it has `REASON`" """
    assert Shop().users == []


@skip("")
def test_empty_reason() -> None:
    """REFUSED before the run: the reason is empty."""
    assert Shop().users == []


@skip_if(True, f"computed {sys.platform}")
def test_reason_by_fstring() -> None:
    """REFUSED before the run: an f-string is no literal."""
    assert Shop().users == []
