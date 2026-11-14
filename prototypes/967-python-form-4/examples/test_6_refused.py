"""6: what the runner refuses before anything runs. A fault of a case stops that case; a fault of a test stops its cases."""

from app import slugify
from cases import case


def test_slug(text: str, expected: str) -> None:
    slug = slugify(text)
    assert slug == expected


fine     = case(test_slug, text="a b", expected="a-b")
typo     = case(test_slug, txt="a b", expected="a-b")  # type: ignore[call-arg]  # ty: ignore[unknown-argument, missing-argument]
computed = case(test_slug, text="a b", expected=slugify("a b"))
two      = case(test_slug, txt="a b", expected=slugify("a b"))  # type: ignore[call-arg]  # ty: ignore[unknown-argument, missing-argument]
by_place = case(test_slug, "a b", "a-b")
again    = case(test_slug, text="x", expected="x")
again    = case(test_slug, text="y", expected="y")
case(test_slug, text="z", expected="z")
built    = [case(test_slug, text=t, expected=t) for t in "pq"]


def test_branchy(text: str, expected: str) -> None:
    slug = slugify(text) if text else ""
    try:
        slug = slugify(text)
    except ValueError:
        slug = ""
    assert slug == expected


some = case(test_branchy, text="a", expected="a")


def test_checks_badly(text: str) -> None:
    slug = slugify(text)
    assert slug
    assert len(slug) > 1
    assert "a-b" == slug
    assert text == "a b"


badly = case(test_checks_badly, text="a b")


def test_reads_an_input(text: str) -> None:
    slugify(text)
    assert text == "a b"


only_input = case(test_reads_an_input, text="a b")


def test_checks_nothing() -> None:
    slugify("a b")


def test_twice() -> None:
    slug = slugify("a b")
    assert slug == "a-b"


def test_twice() -> None:  # noqa: F811
    slug = slugify("c d")
    assert slug == "c-d"
