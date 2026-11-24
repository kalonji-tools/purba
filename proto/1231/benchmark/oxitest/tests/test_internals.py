"""dateutil's tests/test_internals.py::test_YMD_could_be_day, ported to oxitest.

Apache-2.0 / BSD-3-Clause, dateutil/dateutil@a78cbd81: see LICENSE.dateutil.
"""

import oxitest as oxi
from dateutil.parser._parser import _ymd


@oxi.mark.smoke
def test_YMD_could_be_day():
    ymd = _ymd("foo bar 124 baz")

    ymd.append(2, "M")
    assert ymd.has_month, "a month was appended"
    assert not ymd.has_year, "no year was appended yet"
    assert ymd.could_be_day(4), "4 is a day of any month"
    assert not ymd.could_be_day(-6), "a day is never negative"
    assert not ymd.could_be_day(32), "no month has 32 days"

    # Assumes leap year
    assert ymd.could_be_day(29), "with no year, February 29 may exist"

    ymd.append(1999)
    assert ymd.has_year, "a year was appended"
    assert not ymd.could_be_day(29), "1999 is not a leap year"

    ymd.append(16, "D")
    assert ymd.has_day, "a day was appended"
    assert not ymd.could_be_day(1), "the day is already set"

    ymd = _ymd("foo bar 124 baz")
    ymd.append(1999)
    assert ymd.could_be_day(31), "with no month, any day up to 31 may exist"
