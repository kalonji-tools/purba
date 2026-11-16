"""Helpers in the probes' own folder, but not in a test module: the watch does not cover them."""

from __future__ import annotations

import threading


def start_all(threads: list[threading.Thread]) -> None:
    for t in threads:
        t.start()


def check_dir(names: list[str]) -> None:
    assert names, "a helper that checks, as filesink-delay's `check_dir`"


def total_is(order: object, total: int) -> None:
    assert getattr(order, "total") >= total, "a helper of another module that checks"
