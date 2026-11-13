"""ex2: an expected warning and an expected raise; an assay may share a name with an input."""

from app import Money, parse_amount
from form_below import experiment


@experiment
def amount_of(text: str, currency: str | None) -> Money:
    return parse_amount(text, currency)


@amount_of.assay
def cents(product: Money) -> int:
    return product.cents


@amount_of.assay
def currency(product: Money) -> str:
    return product.currency


amount_of.case("euros", text="12.50", currency="EUR").expect(cents=1250, currency="EUR")
amount_of.case("legacy", text="12.50", currency=None).warns(DeprecationWarning, "no currency").expect(cents=1250, currency="USD")
amount_of.case("negative", text="-5", currency="EUR").raises(ValueError, "below zero")
amount_of.case("rounding", text="0.005", currency="EUR").expect(cents=0, currency="EUR")  # warns, unexpected
