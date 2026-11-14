"""ex5: what the parse refuses before anything runs, and one refusal at run time."""

from app import Money, parse_amount, slugify
from form_klasscall import Experiment, case


class Nested(Experiment):
    def act(self, text: str) -> str:
        return slugify(text.strip())

    a = case(act, text="Hi").expect(product="hi")


class Literal(Experiment):
    def act(self, text: str) -> str:
        return slugify(text, "_")  # passes under --literals

    a = case(act, text="a b").expect(product="a_b")


class TypoKey(Experiment):
    def act(self, text: str) -> str:
        return slugify(text)

    a = case(act, text="Hi").expect(slgu="hi")


class ProductOnRaise(Experiment):
    def act(self, text: str) -> str:
        return slugify(text)

    a = case(act, text=" ").raises(ValueError, "empty").expect(product="")


class ReadsLiteral(Experiment):
    def act(self, text: str) -> str:
        return slugify(text)

    def echoed(self, text: str) -> str:
        return text

    a = case(act, text="Hi").expect(product="hi", echoed="Hi")


class MissingValue(Experiment):
    def act(self, text: str, currency: str) -> Money:
        return parse_amount(text, currency)

    def cents(self, product: Money) -> int:
        return product.cents

    def unit(self, product: Money) -> str:
        return product.currency

    a = case(act, text="12.50", currency="EUR").expect(cents=1250)


class ReadingNotLiteral(Experiment):
    def act(self, text: str, currency: str) -> Money:
        return parse_amount(text, currency)

    a = case(act, text="12.50", currency="EUR").expect(product="12.50 EUR")
