seen = []


def check():
    if not seen:
        assert seen == []
    return True


# purba: suppress[assert-in-reading] this case shows the departure
def test_suppressed():
    assert seen.append(1) is None
    assert check() is True
