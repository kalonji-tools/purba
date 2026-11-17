# purba: suppress[not-one-comparison] this case shows the departure
def test_suppressed():
    total = 1 + 1
    assert total == 2 and total > 0
