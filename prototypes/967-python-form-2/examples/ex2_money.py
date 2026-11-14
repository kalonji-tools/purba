"""ex2: an expected warning, an expected raise, and a warning that no case expects."""

from app import Money, parse_amount
from experiment import experiment


@experiment
def amount_of(text: str, currency: str | None) -> Money:
    return parse_amount(text, currency)


@amount_of.assay
def cents(product: Money) -> int:
    return product.cents


@amount_of.assay
def currency(product: Money) -> str:
    return product.currency


euros    = amount_of(text="12.50", currency="EUR").expect(cents=1250, currency="EUR")
legacy   = amount_of(text="12.50", currency=None).warns(DeprecationWarning, match="no currency").expect(cents=1250, currency="USD")
negative = amount_of(text="-5", currency="EUR").raises(ValueError, match="below zero")
rounding = amount_of(text="0.005", currency="EUR").expect(cents=0, currency="EUR")  # warns "rounded to a cent": no case expects it
