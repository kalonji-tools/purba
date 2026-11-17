# purba: suppress[assert-before-expectation] this case shows the departure
def test_suppressed():
    total = 1
    assert total == 1
    total = 2
    assert total == 2
