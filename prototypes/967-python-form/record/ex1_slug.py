"""ex1: a pure function. The product is its own reading; `sep` is a field with a default."""

from app import slugify
from form_record import Experiment


class SlugOf(Experiment[str]):
    text: str
    sep: str = "-"

    def act(self) -> str:
        return slugify(self.text, self.sep)


plain      = SlugOf(text="Hello World").expect(product="hello-world")
spaces     = SlugOf(text="  many   spaces ").expect(product="many-spaces")
accents    = SlugOf(text="Crème Brûlée").expect(product="creme-brulee")  # fails on purpose
underscore = SlugOf(text="a b", sep="_").expect(product="a_b")
empty      = SlugOf(text="   ").raises(ValueError, "empty")
