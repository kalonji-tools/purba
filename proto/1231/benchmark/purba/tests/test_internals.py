import purba

from dateutil.parser._parser import _ymd


# Port of dateutil's test_YMD_could_be_day. purba lets a test check one
# state, so the original's four states are four tests; each carries the tag.


@purba.tag("smoke")
def test_YMD_could_be_day_month():
    ymd = _ymd('foo bar 124 baz')

    ymd.append(2, 'M')
    assert ymd.has_month is True
    assert not ymd.has_year
    assert ymd.could_be_day(4) is True
    assert not ymd.could_be_day(-6)
    assert not ymd.could_be_day(32)

    # Assumes leap year
    assert ymd.could_be_day(29) is True


@purba.tag("smoke")
def test_YMD_could_be_day_month_year():
    ymd = _ymd('foo bar 124 baz')

    ymd.append(2, 'M')
    ymd.append(1999)
    assert ymd.has_year is True
    assert not ymd.could_be_day(29)


@purba.tag("smoke")
def test_YMD_could_be_day_month_year_day():
    ymd = _ymd('foo bar 124 baz')

    ymd.append(2, 'M')
    ymd.append(1999)
    ymd.append(16, 'D')
    assert ymd.has_day is True
    assert not ymd.could_be_day(1)


@purba.tag("smoke")
def test_YMD_could_be_day_year():
    ymd = _ymd('foo bar 124 baz')
    ymd.append(1999)
    assert ymd.could_be_day(31) is True
