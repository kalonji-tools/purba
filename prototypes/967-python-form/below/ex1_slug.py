"""ex1: a pure function. The product is its own reading; `sep` is a default of the act."""

from app import slugify
from form_below import experiment


@experiment
def slug_of(text: str, sep: str = "-") -> str:
    return slugify(text, sep)


slug_of.case("plain", text="Hello World").expect(product="hello-world")
slug_of.case("spaces", text="  many   spaces ").expect(product="many-spaces")
slug_of.case("accents", text="Crème Brûlée").expect(product="creme-brulee")  # fails on purpose
slug_of.case("underscore", text="a b", sep="_").expect(product="a_b")
slug_of.case("empty", text="   ").raises(ValueError, "empty")
