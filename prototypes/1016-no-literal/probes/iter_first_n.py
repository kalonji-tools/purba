import itertools
def test_x():
    it = itertools.count()
    assert list(itertools.islice(it, 3)) == [0, 1, 2]
