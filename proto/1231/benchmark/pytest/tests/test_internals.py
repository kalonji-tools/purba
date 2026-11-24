import pytest

from dateutil.parser._parser import _ymd


@pytest.mark.smoke
def test_YMD_could_be_day():
    ymd = _ymd('foo bar 124 baz')

    ymd.append(2, 'M')
    assert ymd.has_month
    assert not ymd.has_year
    assert ymd.could_be_day(4)
    assert not ymd.could_be_day(-6)
    assert not ymd.could_be_day(32)

    # Assumes leap year
    assert ymd.could_be_day(29)

    ymd.append(1999)
    assert ymd.has_year
    assert not ymd.could_be_day(29)

    ymd.append(16, 'D')
    assert ymd.has_day
    assert not ymd.could_be_day(1)

    ymd = _ymd('foo bar 124 baz')
    ymd.append(1999)
    assert ymd.could_be_day(31)
