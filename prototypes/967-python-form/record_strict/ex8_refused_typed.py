"""ex8: the refusals agreed on 2026-11-14, and the checks of a declared `expect`."""

from typing import Self

from app import Money, parse_amount, slugify
from form_record_strict import UNREAD, TypedExperiment


class NoExpect(TypedExperiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)

    def size(self) -> int:
        return len(self.product)


b1 = NoExpect(text="Hi").expect(size=2)  # ty: no attribute `expect`


class Clash(TypedExperiment[Money]):
    text: str
    currency: str

    def act(self) -> Money:
        return parse_amount(self.text, self.currency)

    def currency(self) -> str:  # the field is obscured, says pyright
        return self.product.currency

    def expect(self, *, currency: str = UNREAD) -> Self:
        return self


b2 = Clash(text="1", currency="EUR").expect(currency="EUR")


class OutOfStep(TypedExperiment[Money]):
    text: str
    currency: str

    def act(self) -> Money:
        return parse_amount(self.text, self.currency)

    def cents(self) -> int:
        return self.product.cents

    def unit(self) -> str:
        return self.product.currency

    def expect(self, *, cents: str = UNREAD, unti: str = UNREAD) -> Self:
        return self


b3 = OutOfStep(text="1", currency="EUR").expect(cents="100", unit="EUR")  # ty: no keyword `unit`


class Defaults(TypedExperiment[None]):
    names: list[str]
    name: str

    def act(self) -> None:
        return self.names.append(self.name)

    def seen(self) -> list[str]:
        return self.names

    def expect(self, *, seen: list[str] = UNREAD, product: None) -> Self:
        return self


b4 = Defaults(names=[], name="ann").expect(seen=["ann"], product=None)


class Twice(TypedExperiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)

    def expect(self, *, product: str = UNREAD) -> Self:
        return self


same = Twice(text="a b").expect(product="a-b")
same = Twice(text="c d").expect(product="c-d")  # the first `same` is lost
