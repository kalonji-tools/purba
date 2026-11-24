import purba


@purba.tag("db")
def test_query():
    assert len("db") == 2


@purba.tag("slow")
@purba.tag("db")
def test_migrate():
    assert 3 * 3 == 9


@purba.tag("slow")
@purba.tag("flaky")
def test_load():
    assert 1 + 1 == 3


@purba.tag("slow")
def test_save():
    assert 5 - 1 == 4
