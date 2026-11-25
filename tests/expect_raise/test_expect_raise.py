class Closed(Exception):
    pass


def lend(shelf, title):
    raise Closed("the library is closed")


def keep(shelf, title):
    return title


def test_pass():
    shelf = ["Dune"]
    try:
        lend(shelf, "Dune")
    except Closed:
        pass
    assert shelf == ["Dune"]


def test_ellipsis():
    shelf = ["Dune"]
    try:
        lend(shelf, "Dune")
    except Closed:
        ...
    assert shelf == ["Dune"]


def test_tuple():
    shelf = ["Dune"]
    try:
        lend(shelf, "Dune")
    except (Closed, KeyError):
        pass
    assert shelf == ["Dune"]


def test_bare():
    shelf = ["Dune"]
    try:
        lend(shelf, "Dune")
    except:
        pass
    assert shelf == ["Dune"]


def test_two_statements():
    shelf = ["Dune"]
    try:
        shelf.append("Emma")
        lend(shelf, "Dune")
    except Closed:
        pass
    assert shelf == ["Dune", "Emma"]


def test_message():
    got = None
    try:
        lend(["Dune"], "Dune")
    except Closed as e:
        got = str(e)
    assert got == "the library is closed"


def test_quiet():
    got = None
    try:
        keep(["Dune"], "Dune")
    except Closed as e:
        got = str(e)
    assert got == "the library is closed"


# purba: suppress[type-alone] the type is the whole contract of lend
def test_suppressed():
    shelf = ["Dune"]
    try:
        lend(shelf, "Dune")
    except Closed:
        pass
    assert shelf == ["Dune"]
