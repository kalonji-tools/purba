"""Helpers in the probes' own folder: the test's own code, so a path in one that the test calls is part of the act."""

from __future__ import annotations

import threading


def start_all(threads: list[threading.Thread]) -> None:
    for t in threads:
        t.start()


def check_dir(names: list[str]) -> None:
    assert names, "a helper that checks, as filesink-delay's `check_dir`"
