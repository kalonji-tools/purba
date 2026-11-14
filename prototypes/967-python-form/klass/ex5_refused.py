"""ex5: what the parse refuses before anything runs, and one refusal at run time."""

from app import Money, parse_amount, slugify
from form_klass import Experiment, case, raises


class Nested(Experiment):
    def act(self, text: str) -> str:
        return slugify(text.strip())

    a = case(text="Hi", product="hi")


class Literal(Experiment):
    def act(self, text: str) -> str:
        return slugify(text, "_")  # passes under --literals

    a = case(text="a b", product="a_b")


class TypoKey(Experiment):
    def act(self, text: str) -> str:
        return slugify(text)

    a = case(text="Hi", slgu="hi")


class ProductOnRaise(Experiment):
    def act(self, text: str) -> str:
        return slugify(text)

    a = case(text=" ", product="", raises=raises(ValueError, "empty"))


class ReadsLiteral(Experiment):
    def act(self, text: str) -> str:
        return slugify(text)

    def echoed(self, text: str) -> str:
        return text

    a = case(text="Hi", product="hi", echoed="Hi")


class MissingValue(Experiment):
    def act(self, text: str, currency: str) -> Money:
        return parse_amount(text, currency)

    def cents(self, product: Money) -> int:
        return product.cents

    def unit(self, product: Money) -> str:
        return product.currency

    a = case(text="12.50", currency="EUR", cents=1250)


class ReadingNotLiteral(Experiment):
    def act(self, text: str, currency: str) -> Money:
        return parse_amount(text, currency)

    a = case(text="12.50", currency="EUR", product="12.50 EUR")
