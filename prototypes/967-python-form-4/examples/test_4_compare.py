"""4: how a reading is compared: `==` by type, then by value; any other single operator as Python does."""

from app import adult_count, slugify
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


def test_operators() -> None:
    slug = slugify("Big Red Dog")
    assert "red" in slug
    assert len(slug) < 20
    assert slug != "big red dog"
    assert slug.find("cat") == -1
    assert "big-" == slug[:4], "the expected value may stand on either side"
