"""Stages that test_far.py takes. Run on its own, or as the stages of test_far.py, each case shows
the same outcome: every rule applies to a stage of another module."""

from __future__ import annotations

from dataclasses import dataclass

from cases import case, use
from shop import Shop


@dataclass
class Member:
    shop: Shop
    name: str


def test_member(name: str, users: list[str], shop: Shop = use(Shop)) -> Member:
    """member PASS, wrong_member FAIL."""
    signed = shop.sign_up(name)
    assert shop.users == users
    return Member(shop, signed)


member = case(test_member, name="ann", users=["ann"])
wrong_member = case(test_member, name="bob", users=["nobody"])


def shown(name: str, vip: bool) -> str:
    if vip:
        return name.upper()
    return name


def test_vip_member(name: str, shop: Shop = use(Shop)) -> Member:
    """REFUSED: the `if` of `shown`, a helper of this module, goes one way. Says "the branch at
    test_far_home.py:30 (shown, reached from line 38 of the test) went one way" """
    signed = shop.sign_up(shown(name, True))
    assert shop.users == ["CY"]
    return Member(shop, signed)


vip_member = case(test_vip_member, name="cy")
