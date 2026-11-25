import json
import warnings


def key(x):
    if x < 0:
        return -x
    return x


def test_private():
    got = json._default_encoder.indent
    assert got is None


def test_empty():
    cart = []
    assert not cart


def test_kind():
    got = 1
    assert isinstance(got, str)


def test_prefix():
    got = "ann-lee"
    assert got.startswith("ann")


def test_bare():
    got = [1]
    assert got


def test_try():
    seen = []
    try:
        int("1")
    except ValueError:
        seen.append("handled")
    assert seen == ["handled"]


def test_filters():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        got = 1
    assert got == 1


def test_callback():
    got = sorted([3, 1, 2], key=key)
    assert got == [1, 2, 3]


def test_act_assert():
    got = [1]
    assert got == [1]
    got.append(2)
    assert got == [1, 2]


def test_input(expected=3):
    got = 1 + 2
    assert got == expected
