"""#1015, change 5: a private name that the body of a test reads, with its nested defs, lambdas and
classes, is reported under the case, never refused. A name the test binds itself is its own, and
an import at module level is not a test's body."""

from __future__ import annotations

import shop as shop_module
from shop import Shop, _log  # an import at module level: not reported


def test_reads_private_names() -> None:
    """PASS, with a report for each private name, once a line. Says "⚠ line 15 reads the private
    name _log" Says "⚠ line 16 reads the private name _log" Says "⚠ line 17 reads the private
    name _flush" """
    log = _log
    same = shop_module._log
    later = lambda: Shop()._flush  # noqa: E731  # ty: ignore[unresolved-attribute]
    shop = Shop()
    shop.sign_up("ann")
    assert log.__name__ == "_log"
    assert same.__name__ == "_log"
    assert later is not None
    assert shop.users == ["ann"]


def test_private_in_a_check_and_a_class() -> None:
    """FAIL: the verdict is the code's; the reports still show. Says "⚠ line 34 reads the private
    name _secret" Says "⚠ line 37 reads the private name _log" """

    class Box:
        _secret = 1

        def __init__(self) -> None:
            self.value = Box._secret

    shop = Shop()
    assert shop_module._log is not None
    assert Box().value == 1
    assert shop.users == ["bob"]


def test_own_names_are_not_private(_reset: int = 0) -> None:
    """PASS, and no report: `_reset`, `_total` and `_` are the test's own, and `__name__` is a
    dunder. Says "✓ test_own_names_are_not_private › _total == 1" """
    _total = _reset + 1
    _ = Shop.__name__
    assert _total == 1
    assert Shop().users == []


def test_private_and_refused(name: str = "") -> None:
    """REFUSED: the `or` goes one way; the report of the private name does not change it. Says
    "⚠ line 55 reads the private name _log" """
    shown = name or "anonymous"
    _log(shown)
    assert shown == "anonymous"
