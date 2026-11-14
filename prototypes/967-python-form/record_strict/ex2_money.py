"""ex2: an expected warning and an expected raise. A field and an assay cannot share a name."""

from typing import Self

from app import Money, parse_amount
from form_record_strict import UNREAD, Experiment


class AmountOf(Experiment[Money]):
    text: str
    currency: str | None

    def act(self) -> Money:
        return parse_amount(self.text, self.currency)

    def cents(self) -> int:
        return self.product.cents

    def unit(self) -> str:
        return self.product.currency

    def expect(self, *, cents: int = UNREAD, unit: str = UNREAD) -> Self:
        return self


euros    = AmountOf(text="12.50", currency="EUR").expect(cents=1250, unit="EUR")
legacy   = AmountOf(text="12.50", currency=None).warns(DeprecationWarning, "no currency").expect(cents=1250, unit="USD")
negative = AmountOf(text="-5", currency="EUR").raises(ValueError, "below zero")
rounding = AmountOf(text="0.005", currency="EUR").expect(cents=0, unit="EUR")  # warns, unexpected
