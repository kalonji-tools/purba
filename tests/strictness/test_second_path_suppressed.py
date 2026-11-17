# purba: suppress[second-path] this case shows the departure
def test_suppressed():
    total = 1
    if total:
        total = 2
    assert total == 2
