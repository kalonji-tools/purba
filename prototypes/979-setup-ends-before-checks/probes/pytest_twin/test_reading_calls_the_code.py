"""Probe from the colorama port: a check whose reading calls the code under test.

In pytest, the second assert reads the state the first assert's call left. Here each check but the last runs in a
fork, so the second reads the state from before the call. The code under test calls `flush`, so pytest fails this
test; the runner passes it, and nothing on the screen says so.
"""

from unittest.mock import MagicMock



def wrap(stream: MagicMock) -> bool:
    stream.flush()  # the code under test touches the stream
    return True


def test_reading_calls_the_code():
    stream = MagicMock()
    assert wrap(stream) is True
    assert stream.flush.called is False


