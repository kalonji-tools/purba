class Closed(Exception):
    pass


def lend(shelf):
    raise Closed("closed")


# purba: suppress[type-alone] this case shows the departure
def test_suppressed():
    shelf = ["Dune"]
    try:
        lend(shelf)
    except Closed:
        pass
    assert shelf == ["Dune"]
