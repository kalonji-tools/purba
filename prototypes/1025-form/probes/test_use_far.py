"""Stages of another test module. The runner loads that module rewritten, and runs each stage
this file takes in this file's run, with every rule; a stage that fails blocks the test."""

from __future__ import annotations

from cases import use
from test_use_stage import Filled, ann_books, wrong


def test_far_checkout(filled: Filled = use(ann_books)) -> None:
    """PASS: `ann` and `ann_books` run here first, each with its checks."""
    order = filled.shop.checkout(filled.user)
    assert order.total == 2400


def test_far_after_wrong(filled: Filled = use(wrong)) -> None:
    """BLOCKED. Says "it takes the product of wrong (test_use_stage.py:44), which failed" """
    order = filled.shop.checkout(filled.user)
    assert order.total == 1200
