"""Every check reads in place, in order, and each later check is also read alone. A check whose
two verdicts differ is refused: a reading before it changed what it reads."""

from __future__ import annotations

import io
from unittest.mock import MagicMock

from undertest import wrap


def test_reading_calls_the_code():
    """REFUSED: the first reading calls wrap(), which flushes, so the second check reads True."""
    stream = MagicMock()
    assert wrap(stream) is True
    assert stream.flush.called is False


def test_reading_consumes_a_stream():
    """REFUSED: the first read empties the stream, so the second check reads "" after it, and "x"
    alone."""
    stream = io.StringIO("x")
    assert stream.read() == "x"
    assert stream.read() == "x"


def test_pure_readings():
    """PASS: neither reading changes anything."""
    seen = {"a": 1}
    assert seen["a"] == 1
    assert "b" not in seen


def test_the_act_flushes():
    """PASS: the call is in the act, where it belongs, and both checks read what it made."""
    stream = MagicMock()
    wrapped = wrap(stream)
    assert wrapped is True
    assert stream.flush.called is True
