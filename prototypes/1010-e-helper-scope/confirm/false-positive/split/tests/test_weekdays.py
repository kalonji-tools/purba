"""In the shape of schedule's test_move_to_next_weekday_today."""

import datetime

import weekdays


def test_move_to_next_weekday_today():
    monday = datetime.datetime(2024, 5, 13, 10, 27, 54)
    moved = weekdays.move_to_next_weekday(monday, "monday")
    assert moved.day == 13
    assert moved.hour == 10
