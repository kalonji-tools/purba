"""ex5: what the parse refuses before anything runs, and one refusal at run time."""

from app import Money, parse_amount, slugify
from form_record import Experiment


class Nested(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text.strip())


a1 = Nested(text="Hi").expect(product="hi")


class Literal(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text, "_")  # passes under --literals


a2 = Literal(text="a b").expect(product="a_b")


class TypoKey(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)


a3 = TypoKey(text="Hi").expect(slgu="hi")


class ProductOnRaise(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)


a4 = ProductOnRaise(text=" ").raises(ValueError, "empty").expect(product="")


class ReadsLiteral(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)

    def echoed(self) -> str:
        return self.text


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


a6 = MissingValue(text="12.50", currency="EUR").expect(cents=1250)


class ReadingNotLiteral(Experiment[Money]):
    text: str
    currency: str

    def act(self) -> Money:
        return parse_amount(self.text, self.currency)


a7 = ReadingNotLiteral(text="12.50", currency="EUR").expect(product="12.50 EUR")
