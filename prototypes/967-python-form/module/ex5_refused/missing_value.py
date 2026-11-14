from app import Money, parse_amount
from form_module import case


def act(text: str, currency: str) -> Money:
    return parse_amount(text, currency)


def cents(product: Money) -> int:
    return product.cents


def unit(product: Money) -> str:
    return product.currency


a = case(text="12.50", currency="EUR", cents=1250)
