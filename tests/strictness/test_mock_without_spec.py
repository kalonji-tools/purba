from unittest import mock


def test_refused():
    stand_in = mock.Mock()
    assert stand_in.call_count == 0
