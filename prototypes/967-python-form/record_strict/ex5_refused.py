"""ex5: what the parse refuses before anything runs. Lines marked `# ty` are seen by a type checker too."""

from typing import Self

from app import Money, parse_amount, slugify
from form_record_strict import UNREAD, Experiment


class Nested(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text.strip())

    def expect(self, *, product: str = UNREAD) -> Self:
        return self


a1 = Nested(text="Hi").expect(product="hi")


class Literal(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text, "_")  # passes under --literals

    def expect(self, *, product: str = UNREAD) -> Self:
        return self


a2 = Literal(text="a b").expect(product="a_b")


class TypoKey(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)

    def expect(self, *, product: str = UNREAD) -> Self:
        return self


a3 = TypoKey(text="Hi").expect(prodcut="hi")  # ty


class ProductOnRaise(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)

    def expect(self, *, product: str = UNREAD) -> Self:
        return self


a4 = ProductOnRaise(text=" ").raises(ValueError, "empty").expect(product="")


class ReadsLiteral(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)

    def echoed(self) -> str:
        return self.text

    def expect(self, *, echoed: str, product: str = UNREAD) -> Self:
        return self


a5 = ReadsLiteral(text="Hi").expect(product="hi", echoed="Hi")


class MissingValue(Experiment[Money]):
    text: str
    currency: str

    def act(self) -> Money:
        return parse_amount(self.text, self.currency)

    def cents(self) -> int:
        return self.product.cents

    def unit(self) -> str:
        return self.product.currency

    def expect(self, *, cents: int = UNREAD, unit: str = UNREAD) -> Self:
        return self


a6 = MissingValue(text="12.50", currency="EUR").expect(cents=1250)


class ReadingNotLiteral(Experiment[Money]):
    text: str
    currency: str

    def act(self) -> Money:
        return parse_amount(self.text, self.currency)

    def expect(self, *, product: Money = UNREAD) -> Self:
        return self


a7 = ReadingNotLiteral(text="12.50", currency="EUR").expect(product="12.50 EUR")  # ty
