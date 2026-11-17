seen = []


# purba: suppress[readings-differ] this case shows the departure
def test_suppressed():
    assert seen.append(1) is None
    assert seen == [1]
