"""The watch of the act covers the code of the test module only, wherever other code sits, and a
`@contextmanager` function of the test module that a `with` of the test enters."""

from __future__ import annotations

from collections.abc import Iterator
from contextlib import contextmanager

from flat import Box, opened_box
from shop import Shop


def test_folder_code_is_not_watched() -> None:
    """PASS: Box's `if` goes one way, but flat.py sits in this folder and is no test module."""
    box = Box(None)
    assert box.v == 0


def test_contextmanager_of_another_file() -> None:
    """PASS: the `if` of `opened_box` goes one way, but it sits in flat.py."""
    with opened_box(None) as box:
        box.v += 1
    assert box.v == 1


@contextmanager
def loud(shop: Shop, on: bool) -> Iterator[Shop]:
    if on:
        shop.sign_up("LOUD")
    yield shop


def test_contextmanager_is_watched() -> None:
    """REFUSED: the `if` of `loud`, a `@contextmanager` of this module that the `with` enters,
    goes one way. Says "the branch at test_watch.py:28 (loud, reached from line 38 of the test)
    went one way" """
    shop = Shop()
    with loud(shop, on=False) as entered:
        entered.sign_up("ann")
    assert shop.users == ["ann"]


@contextmanager
def signed_in(shop: Shop, name: str) -> Iterator[Shop]:
    shop.sign_up(name)
    yield shop
    shop.carts[name].clear()


def test_contextmanager_of_one_path() -> None:
    """PASS: each line of `signed_in` runs, its undo too, and it has no branch."""
    shop = Shop()
    with signed_in(shop, "ann") as entered:
        entered.add("ann", "PEN", 1)
    assert shop.carts["ann"] == []


@contextmanager
def checked(shop: Shop) -> Iterator[Shop]:
    yield shop
    assert shop.users


def test_assert_in_a_contextmanager() -> None:
    """REFUSED: the undo of `checked` asserts, in the act. Says "an assert ran in the act at
    test_watch.py:61 (checked, reached from line 68 of the test)" """
    shop = Shop()
    with checked(shop) as entered:
        entered.sign_up("ann")
    assert shop.users == ["ann"]
