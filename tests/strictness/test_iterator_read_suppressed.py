# purba: suppress[iterator-read] this case shows the departure
def test_suppressed():
    it = iter([1, 2])
    assert list(it) == [1, 2]
