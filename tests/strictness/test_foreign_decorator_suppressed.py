def keep(f):
    return f


# purba: suppress[foreign-decorator] this case shows the departure
@keep
def test_suppressed():
    assert 1 + 1 == 2
