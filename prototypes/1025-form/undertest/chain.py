"""Code under test for the chain probes. `act` writes each call to the file that FORM_ACTS names,
and `count` reads it, so a check sees how often an act ran in one run of the runner."""

from __future__ import annotations

import os
import tempfile
import threading


def act(name: str) -> str:
    with open(os.environ["FORM_ACTS"], "a") as f:
        f.write(name + "\n")
    return name


def count(name: str) -> int:
    with open(os.environ["FORM_ACTS"]) as f:
        return f.read().splitlines().count(name)


class Worker:
    """A thread that stays alive until it is stopped."""

    def __init__(self, name: str) -> None:
        self.stop = threading.Event()
        self.thread = threading.Thread(target=self.stop.wait, name=name, daemon=True)
        self.thread.start()


class Journal:
    """A temporary file that stays open."""

    def __init__(self, first: str) -> None:
        self.file = tempfile.TemporaryFile("w+")
        self.file.write(first + "\n")
        self.file.flush()

    def lines(self) -> list[str]:
        self.file.seek(0)
        return self.file.read().splitlines()
