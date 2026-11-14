"""4: how a reading is compared: by type, then by value."""

from app import adult_count
from cases import case


class Name(str):
    """A subclass of str, as loguru's Message is."""


def test_name() -> None:
    name = Name("ann")
    assert name == "ann", "a subclass of str reads as a str"


def test_adults(ages: list[int], expected: int) -> None:
    count = adult_count(ages)
    assert count == expected


two = case(test_adults, ages=[20, 30, 5], expected=2)
one = case(test_adults, ages=[20], expected=1)  # fails: the code returns True for one age, and True is not 1
