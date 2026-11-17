# purba: suppress[no-item] this case shows the departure
def test_suppressed():
    xs = []
    assert all(x > 0 for x in xs)
