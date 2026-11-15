"""A stage that failed, or that the runner refused, blocks each later stage. The blocked line names
the stage, the line of its case, and what it did."""

from __future__ import annotations

from cases import case, use
from chain import Worker
from shop import Shop


def test_stage(name: str, users: list[str]) -> Shop:
    """good PASS, wrong FAIL."""
    shop = Shop()
    shop.sign_up(name)
    assert shop.users == users
    return shop


good = case(test_stage, name="ann", users=["ann"])
wrong = case(test_stage, name="ann", users=["bob"])


def test_after_good(shop: Shop = use(good)) -> None:
    """PASS: its stage passed."""
    shop.sign_up("cy")
    assert shop.users == ["ann", "cy"]


def test_after_wrong(shop: Shop = use(wrong)) -> Shop:
    """BLOCKED. Says "it takes the product of wrong (test_blocked.py:20), which failed" """
    shop.sign_up("cy")
    assert shop.users == ["ann", "cy"]
    return shop


after_wrong = case(test_after_wrong)


def test_two_down(shop: Shop = use(after_wrong)) -> None:
    """BLOCKED. Says "it takes the product of after_wrong (test_blocked.py:36), which was
    blocked" """
    shop.sign_up("dee")
    assert shop.users == ["ann", "cy", "dee"]


def test_one_way(name: str, loud: bool = False) -> Shop:
    """REFUSED while it runs: the `x if c else y` goes one way."""
    shop = Shop()
    shop.sign_up(name.upper() if loud else name)
    assert shop.users == [name]
    return shop


one_way = case(test_one_way, name="dee")


def test_after_one_way(shop: Shop = use(one_way)) -> None:
    """BLOCKED. Says "it takes the product of one_way (test_blocked.py:54), which was refused" """
    shop.sign_up("eve")
    assert shop.users == ["dee", "eve"]


def test_computes(name: str) -> Shop:
    """REFUSED before the run: its expected value is computed."""
    shop = Shop()
    shop.sign_up(name)
    assert shop.users == [name.upper()]
    return shop


computed = case(test_computes, name="fay")


def test_after_computes(shop: Shop = use(computed)) -> None:
    """BLOCKED. Says "it takes the product of computed (test_blocked.py:71), which was refused" """
    shop.sign_up("gus")
    assert shop.users == ["fay", "gus"]


def test_threaded_stage(name: str) -> Worker:
    """FAIL: a stage whose chain is replayed, because its act leaves a thread alive."""
    worker = Worker(name)
    assert worker.thread.name == "nobody"
    return worker


threaded_wrong = case(test_threaded_stage, name="worker")


def test_after_threaded_wrong(worker: Worker = use(threaded_wrong)) -> None:
    """BLOCKED, under the replay too. Says "it takes the product of threaded_wrong
    (test_blocked.py:87), which failed" """
    assert worker.thread.is_alive() is True
