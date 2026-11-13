"""ex1: a pure function. The product is its own reading; `sep` is a default of the act."""

from app import slugify
from form_stacked import case, raises


@case("plain",      input=dict(text="Hello World"),      assay=dict(product="hello-world"))
@case("spaces",     input=dict(text="  many   spaces "), assay=dict(product="many-spaces"))
@case("accents",    input=dict(text="Crème Brûlée"),     assay=dict(product="creme-brulee"))  # fails on purpose
@case("underscore", input=dict(text="a b", sep="_"),     assay=dict(product="a_b"))
@case("empty",      input=dict(text="   "),              raises=raises(ValueError, "empty"))
def slug_of(text: str, sep: str = "-") -> str:
    return slugify(text, sep)
