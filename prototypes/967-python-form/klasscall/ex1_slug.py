"""ex1: a pure function. The product is its own reading; `sep` is a default of the act."""

from app import slugify
from form_klasscall import Experiment, case


class SlugOf(Experiment):
    def act(text: str, sep: str = "-") -> str:
        return slugify(text, sep)

    plain      = case(act, text="Hello World").expect(product="hello-world")
    spaces     = case(act, text="  many   spaces ").expect(product="many-spaces")
    accents    = case(act, text="Crème Brûlée").expect(product="creme-brulee")  # fails on purpose
    underscore = case(act, text="a b", sep="_").expect(product="a_b")
    empty      = case(act, text="   ").raises(ValueError, "empty")
