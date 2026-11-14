"""ex1: a pure function. The product is its own reading; `sep` is a default of the act."""

from app import slugify
from form_call import experiment


@experiment
def slug_of(text: str, sep: str = "-") -> str:
    return slugify(text, sep)


plain      = slug_of(text="Hello World").expect(product="hello-world")
spaces     = slug_of(text="  many   spaces ").expect(product="many-spaces")
accents    = slug_of(text="Crème Brûlée").expect(product="creme-brulee")  # fails on purpose
underscore = slug_of(text="a b", sep="_").expect(product="a_b")
empty      = slug_of(text="   ").raises(ValueError, "empty")
