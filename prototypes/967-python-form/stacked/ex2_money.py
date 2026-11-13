"""ex2: an expected warning and an expected raise; an assay may share a name with an input."""

from app import Money, parse_amount
from form_stacked import assay, case, raises, warns


@case("euros",    input=dict(text="12.50", currency="EUR"), assay=dict(cents=1250, currency="EUR"))
@case("legacy",   input=dict(text="12.50", currency=None),  assay=dict(cents=1250, currency="USD"),
      warns=warns(DeprecationWarning, "no currency"))
@case("negative", input=dict(text="-5", currency="EUR"),    raises=raises(ValueError, "below zero"))
@case("rounding", input=dict(text="0.005", currency="EUR"), assay=dict(cents=0, currency="EUR"))  # warns, unexpected
def amount_of(text: str, currency: str | None) -> Money:
    return parse_amount(text, currency)


@assay(amount_of)
def cents(product: Money) -> int:
    return product.cents


@assay(amount_of)
def currency(product: Money) -> str:
    return product.currency
