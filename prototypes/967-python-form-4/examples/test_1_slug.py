"""1: a test with no case, a test with named cases, and an expected raise."""

from app import slugify
from cases import case, raises


def test_two_words() -> None:
    slug = slugify("Hello World")
    assert slug == "hello-world"


def test_slug(text: str, sep: str, expected: str) -> None:
    slug = slugify(text, sep)
    assert slug == expected, "words are lowered and joined by the separator"


plain   = case(test_slug, text="Hello World", sep="-", expected="hello-world")
under   = case(test_slug, text="a b c", sep="_", expected="a_b_c")
accents = case(test_slug, text="Crème Brûlée", sep="-", expected="creme-brulee")  # fails: the code keeps accents


def test_empty(text: str) -> None:
    with raises(ValueError, match="^empty$"):
        slugify(text)


nothing = case(test_empty, text="")
blanks  = case(test_empty, text="   ")
