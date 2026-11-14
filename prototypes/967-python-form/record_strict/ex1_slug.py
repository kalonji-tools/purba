"""ex1: a pure function. `product` is the built-in assay; `expect` types it as the product's type."""

from typing import Self

from app import slugify
from form_record_strict import UNREAD, Experiment


class SlugOf(Experiment[str]):
    text: str
    sep: str = "-"

    def act(self) -> str:
        return slugify(self.text, self.sep)

    def expect(self, *, product: str = UNREAD) -> Self:
        return self


plain      = SlugOf(text="Hello World").expect(product="hello-world")
spaces     = SlugOf(text="  many   spaces ").expect(product="many-spaces")
accents    = SlugOf(text="Crème Brûlée").expect(product="creme-brulee")  # fails on purpose
underscore = SlugOf(text="a b", sep="_").expect(product="a_b")
empty      = SlugOf(text="   ").raises(ValueError, "empty")
