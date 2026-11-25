def test_top():
    assert 1 + 2 == 4


def test_top():
    assert 1 + 2 == 3


if True:
    def test_under():
        assert 1 + 2 == 4


def test_under():
    assert 1 + 2 == 3


# purba: suppress[name-bound-again] the second def replaces a draft
def test_suppressed():
    assert 1 + 2 == 4


def test_suppressed():
    assert 1 + 2 == 3


def test_once():
    assert 1 + 2 == 3


class Box:
    def test_twice(self):
        assert 1 + 2 == 4

    def test_twice(self):
        assert 1 + 2 == 3
