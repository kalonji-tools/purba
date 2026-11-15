"""Mirrors the shape of schedule's `_move_to_next_weekday`: code under test, with one `if`."""

import datetime

WEEKDAYS = ("monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday")


def move_to_next_weekday(moment, weekday):
    days_ahead = WEEKDAYS.index(weekday) - moment.weekday()
    if days_ahead < 0:
        days_ahead += 7
    return moment + datetime.timedelta(days=days_ahead)
