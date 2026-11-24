import purba


def test_add():
    assert 2 * 2 == 4


@purba.tag("slow")
def test_remove():
    assert 2 - 2 == 1
