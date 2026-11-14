"""ex7: a misspelled input, an input of the wrong type, and a chain that feeds the wrong type."""

from typing import Self

from app import slugify
from form_record_strict import UNREAD, Experiment


class SlugOf(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)

    def size(self) -> int:
        return len(self.product)

    def expect(self, *, size: int = UNREAD, product: str = UNREAD) -> Self:
        return self


plain = SlugOf(text="Hello World").expect(product="hello-world", size=11)
typo  = SlugOf(txt="Hello World").expect(product="hello-world", size=11)  # ty
wrong = SlugOf(text=42).expect(product="42", size=2)  # ty


class SlugAgain(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)

    def expect(self, *, product: str = UNREAD) -> Self:
        return self


wrong_again = SlugAgain(text=42).expect(product="42")  # ty
fed_wrong   = SlugAgain(text=plain.size).expect(product="11")  # ty: a step's assay is no value to chain
