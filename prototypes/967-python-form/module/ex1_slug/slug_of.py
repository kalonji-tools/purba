"""ex1: a pure function. The file is the experiment; `sep` is a default of the act."""

from app import slugify
from form_module import case, raises


def act(text: str, sep: str = "-") -> str:
    return slugify(text, sep)


plain      = case(text="Hello World",      product="hello-world")
spaces     = case(text="  many   spaces ", product="many-spaces")
accents    = case(text="Crème Brûlée",     product="creme-brulee")  # fails on purpose
underscore = case(text="a b", sep="_",     product="a_b")
empty      = case(text="   ",              raises=raises(ValueError, "empty"))
