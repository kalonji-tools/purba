from unittest import mock


# purba: suppress[mock-without-spec] this case shows the departure
def test_suppressed():
    stand_in = mock.Mock()
    assert stand_in.call_count == 0
