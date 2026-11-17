seen = []


def check():
    if not seen:
        assert seen == []
    return True


def test_refused():
    assert seen.append(1) is None
    assert check() is True
