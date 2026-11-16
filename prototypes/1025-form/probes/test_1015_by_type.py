"""#1015, change 4: `with raises(T):` or `with warns(T):` with no `match=` is refused before the run,
unless it binds `as caught` and a check reads `caught.value`."""

from __future__ import annotations

import warnings

from cases import raises, warns
from shop import Shop


def old_name() -> None:
    warnings.warn("old_name is gone; use new_name", DeprecationWarning, stacklevel=2)


def test_raises_with_match() -> None:
    """PASS: the message is matched."""
    shop = Shop()
    shop.sign_up("ann")
    with raises(ValueError, match="ann is taken"):
        shop.sign_up("ann")


def test_raises_checked_by_value() -> None:
    """PASS: no `match=`, but a check reads `caught.value`."""
    shop = Shop()
    shop.sign_up("ann")
    with raises(ValueError) as caught:
        shop.sign_up("ann")
    assert str(caught.value) == "ann is taken"


def test_raises_with_another_message() -> None:
    """FAIL: the raise came with another message. Says "expected ValueError matching 'bob'; it
    raised ValueError: ann is taken" """
    shop = Shop()
    shop.sign_up("ann")
    with raises(ValueError, match="bob"):
        shop.sign_up("ann")


def test_warns_checked_by_value() -> None:
    """PASS: a check reads the warning that `caught.value` holds."""
    with warns(DeprecationWarning) as caught:
        old_name()
    assert str(caught.value).startswith("old_name is gone")


def test_raises_by_type_alone() -> None:
    """REFUSED before the run. Says "line 54: `with raises(ValueError):` checks the type alone;
    add `match=r"…"`, or check `caught.value`" """
    shop = Shop()
    shop.sign_up("ann")
    with raises(ValueError):
        shop.sign_up("ann")


def test_warns_by_type_alone() -> None:
    """REFUSED before the run. Says "`with warns(DeprecationWarning):` checks the type alone" """
    with warns(DeprecationWarning):
        old_name()


def test_caught_but_not_read() -> None:
    """REFUSED before the run: `as caught` binds it, and no check reads `caught.value`."""
    shop = Shop()
    shop.sign_up("ann")
    with raises(ValueError) as caught:
        shop.sign_up("ann")
    assert shop.users == ["ann"]
    assert caught is not None


def test_match_none() -> None:
    """REFUSED before the run: `match=None` matches nothing more than the type."""
    shop = Shop()
    shop.sign_up("ann")
    with raises(ValueError, match=None):
        shop.sign_up("ann")
