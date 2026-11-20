def test_concat():
    got = "ab"
    assert got == "a" "b"


def test_fstring():
    got = "a"
    assert got == f"a"


def test_fstring_placeholder():
    got, n = "a1", 1
    assert got == f"a{n}"


def test_negative():
    got = -1
    assert got == -1


def test_dict_splat():
    got, rest = {1: 2}, {}
    assert got == {1: 2, **rest}


def test_not_true():
    got = False
    assert got == (not True)
