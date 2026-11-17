def keep(f):
    return f


@keep
def test_refused():
    assert 1 + 1 == 2
