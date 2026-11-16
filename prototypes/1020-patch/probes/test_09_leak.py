import lending


def test_a_patches():
    lending.Library = None
    assert lending.Library is None


def test_b_reads_real():
    got = lending.borrow("Dune")
    assert got == "lent Dune"
