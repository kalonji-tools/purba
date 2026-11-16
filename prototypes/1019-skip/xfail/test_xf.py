import pytest


@pytest.mark.xfail(reason="bug 12")
def test_lax_still_broken():
    assert 1 + 1 == 3


@pytest.mark.xfail(reason="bug 12")
def test_lax_now_fixed():
    assert 1 + 1 == 2


@pytest.mark.xfail(reason="bug 12", strict=True)
def test_strict_still_broken():
    assert 1 + 1 == 3


@pytest.mark.xfail(reason="bug 12", strict=True)
def test_strict_now_fixed():
    assert 1 + 1 == 2


@pytest.mark.xfail(reason="bug 12", strict=True, raises=TypeError)
def test_strict_wrong_failure():
    raise KeyError("other")
