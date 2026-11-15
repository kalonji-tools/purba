"""INJECTED CONTROL, not corpus evidence (#1010): a product that holds an open file.

A fork copies the file object but shares the file's offset with every other fork, so the
first later stage's read moves the offset for the second. Replay opens a file of its own for
each later stage.
"""

from __future__ import annotations

import tempfile
from typing import IO

from cases import case


def test_open(text: str) -> IO[str]:
    f = tempfile.TemporaryFile("w+")
    f.write(text)
    f.seek(0)
    assert f.tell() == 0
    return f


opened = case(test_open, text="abc")


def test_read(f: IO[str], text: str) -> None:
    got = f.read()
    assert got == text


first_read = case(test_read, f=opened.product, text="abc")
second_read = case(test_read, f=opened.product, text="abc")
