def test_isinstance():
    assert isinstance(1, int)


def test_isinstance_three():
    assert isinstance(1, int, )


def test_startswith_two():
    assert "ab".startswith("a", 0)


def test_startswith_keyword():
    assert "ab".startswith(prefix="a")


def test_star():
    pair = (1, int)
    assert isinstance(*pair)


def test_not_compare():
    got, want = 1, 1
    assert not got == want
