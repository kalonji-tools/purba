"""Stages of another test module. The runner loads test_far_home.py rewritten, as it loads this
file, and runs each stage this file takes in this file's run, with every rule: a refusal while the
stage runs is seen here too, even when this file runs alone."""

from __future__ import annotations

from cases import use
from test_far_home import Member, member, vip_member, wrong_member


def test_buy(signed: Member = use(member)) -> None:
    """PASS: `member` runs here first, with its check."""
    signed.shop.add(signed.name, "BOOK", 2)
    order = signed.shop.checkout(signed.name)
    assert order.total == 2400


def test_after_far_wrong(signed: Member = use(wrong_member)) -> None:
    """BLOCKED. Says "it takes the product of wrong_member (test_far_home.py:26), which failed" """
    order = signed.shop.checkout(signed.name)
    assert order.total == 0


def test_after_far_refused(signed: Member = use(vip_member)) -> None:
    """BLOCKED. Says "it takes the product of vip_member (test_far_home.py:43), which was
    refused" """
    order = signed.shop.checkout(signed.name)
    assert order.total == 0
