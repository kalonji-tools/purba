"""7: what ty and pyright see in a case: a misspelled input, an input of the wrong type, and a missing input.

Each line below carries one mistake on purpose. A wrong type on purpose goes in the act, not on a case line:
declare the input `object`, and the one line of the act that breaks the type carries a coded ignore.
"""

from app import slugify
from cases import case, raises


def test_slug(text: str, expected: str) -> None:
    slug = slugify(text)
    assert slug == expected


misspelled = case(test_slug, txt="a b", expected="a-b")
wrong_type = case(test_slug, text="a b", expected=3)
missing    = case(test_slug, text="a b")


def test_not_a_str(text: object) -> None:
    with raises(AttributeError):
        slugify(text)  # pyright: ignore[reportArgumentType]  # ty: ignore[invalid-argument-type]


number = case(test_not_a_str, text=3)
