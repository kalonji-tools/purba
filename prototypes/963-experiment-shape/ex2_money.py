"""ex2: expected raises and expected warnings live in the table, beside the values."""

from experiment import experiment, raises, warns
from prototype_app import Money, parse_amount


@experiment(
    columns=  ("text",  "currency", "cents", "unit", "raises",                         "warns"),
    euros=    ("12.50", "EUR",      1250,    "EUR",  ...,                              ...),
    legacy=   ("12.50", None,       1250,    "USD",  ...,                              warns(DeprecationWarning, "currency")),
    negative= ("-5",    "EUR",      ...,     ...,    raises(ValueError, "below zero"), ...),
    rounding= ("0.005", "EUR",      1,       "EUR",  ...,                              ...),  # the app warns; the table doesn't expect it
)
def amount_of(text: str, currency: str | None) -> Money:
    return parse_amount(text, currency)


@amount_of.assay
def cents(product: Money) -> int:
    return product.cents


@amount_of.assay
def unit(product: Money) -> str:
    return product.currency
