"""ex2: an expected warning and an expected raise. Keys are flat, so an assay cannot share an input's name."""

from app import Money, parse_amount
from form_module import case, raises, warns


def act(text: str, currency: str | None) -> Money:
    return parse_amount(text, currency)


def cents(product: Money) -> int:
    return product.cents


def unit(product: Money) -> str:  # `currency` is the input's name
    return product.currency


euros    = case(text="12.50", currency="EUR", cents=1250, unit="EUR")
legacy   = case(text="12.50", currency=None,  cents=1250, unit="USD", warns=warns(DeprecationWarning, "no currency"))
negative = case(text="-5",    currency="EUR", raises=raises(ValueError, "below zero"))
rounding = case(text="0.005", currency="EUR", cents=0,    unit="EUR")  # warns, unexpected
