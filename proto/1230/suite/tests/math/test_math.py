import purba


def test_add():
    assert 1 + 1 == 2


@purba.tag("slow")
def test_add_slow():
    assert 2 + 2 == 4


def test_parse():
    assert int("7") == 7
