seen = []


def test_refused():
    assert seen.append(1) is None
    assert seen == [1]
