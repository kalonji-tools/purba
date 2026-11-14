"""ex2: an expected warning and an expected raise; an assay may share a name with an input."""

from app import Money, parse_amount
from form_klasscall import Experiment, case


class AmountOf(Experiment):
    def act(self, text: str, currency: str | None) -> Money:
        return parse_amount(text, currency)

    def cents(self, product: Money) -> int:
        return product.cents

    def currency(self, product: Money) -> str:
        return product.currency

    euros    = case(act, text="12.50", currency="EUR").expect(cents=1250, currency="EUR")
    legacy   = case(act, text="12.50", currency=None).warns(DeprecationWarning, "no currency").expect(cents=1250, currency="USD")
    negative = case(act, text="-5", currency="EUR").raises(ValueError, "below zero")
    rounding = case(act, text="0.005", currency="EUR").expect(cents=0, currency="EUR")  # warns, unexpected
