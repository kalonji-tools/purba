class Closed(Exception):
    pass


def lend(shelf):
    raise Closed("closed")


def test_refused():
    shelf = ["Dune"]
    try:
        lend(shelf)
    except Closed:
        pass
    assert shelf == ["Dune"]
