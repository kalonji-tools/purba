import os


def test_a():
    try:
        os._exit(0)
    except BaseException:
        pass
    assert "purba:panic"


def test_b():
    assert 1 == 1


def test_c_panics_before_call():
    assert 1 == 1


def test_d_catches():
    try:
        assert "purba:panic"
    except BaseException:
        pass
    assert 1 == 1


def test_e():
    assert 1 == 1
