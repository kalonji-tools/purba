"""A fixture may use a fixture: a factory's parameter whose default is `use(other)` gets `other`'s
value of the same case. Each factory runs at most once in a case. A cycle, and a parameter that is no
fixture, are refused before the run."""

from __future__ import annotations

from cases import case, use
from shop import Shop


def shop_with_ann(shop: Shop = use(Shop)) -> Shop:
    shop.sign_up("ann")
    return shop


def test_shared_once(name: str, signed: Shop = use(shop_with_ann), shop: Shop = use(Shop)) -> None:
    """PASS: the test and its fixture share one Shop, so ann and the new user sit in one shop."""
    shop.sign_up(name)
    assert signed.users == ["ann", name]


bob = case(test_shared_once, name="bob")
cy = case(test_shared_once, name="cy")  # a fresh shop again: cy's shop never saw bob


def ring_a(other: object = None) -> object:
    return other


def ring_b(other: object = use(ring_a)) -> object:
    return other


ring_a.__defaults__ = (use(ring_b),)  # a `def` cannot name a later one: only this builds a cycle


def test_cycle(value: object = use(ring_a)) -> None:
    """REFUSED before the run. Says "the fixtures ring_a → ring_b → ring_a form a cycle" """
    shop = Shop()
    shop.sign_up("ann")
    assert shop.users == ["ann"]


def named(name: str) -> Shop:
    shop = Shop()
    shop.sign_up(name)
    return shop


def defaulted(name: str = "ann") -> Shop:
    shop = Shop()
    shop.sign_up(name)
    return shop


def test_parameter_with_no_default(shop: Shop = use(named)) -> None:
    """REFUSED before the run. Says "the fixture named takes name, whose default is no `use(…)`" """
    shop.sign_up("bob")
    assert shop.users == ["ann", "bob"]


def test_parameter_with_a_plain_default(shop: Shop = use(defaulted)) -> None:
    """REFUSED before the run. Says "the fixture defaulted takes name, whose default is no
    `use(…)`" """
    shop.sign_up("bob")
    assert shop.users == ["ann", "bob"]
