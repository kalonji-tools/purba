import sys

if sys:
    # purba: suppress[def-under-block] this case shows the departure
    def test_suppressed():
        assert 1 == 1
