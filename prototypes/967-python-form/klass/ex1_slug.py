"""ex1: a pure function. The product is its own reading; `sep` is a default of the act."""

from app import slugify
from form_klass import Experiment, case, raises


class SlugOf(Experiment):
    def act(self, text: str, sep: str = "-") -> str:
        return slugify(text, sep)

    plain      = case(text="Hello World",      product="hello-world")
    spaces     = case(text="  many   spaces ", product="many-spaces")
    accents    = case(text="Crème Brûlée",     product="creme-brulee")  # fails on purpose
    underscore = case(text="a b", sep="_",     product="a_b")
    empty      = case(text="   ",              raises=raises(ValueError, "empty"))
