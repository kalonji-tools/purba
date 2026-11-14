"""2: an expected warning, an expected raise, and a warning that no test expects."""

from app import parse_amount
from cases import case, raises, warns


def test_amount(text: str, currency: str, cents: int) -> None:
    money = parse_amount(text, currency)
    assert money.cents == cents
    assert money.currency == currency


euros   = case(test_amount, text="12.50", currency="EUR", cents=1250)
rounded = case(test_amount, text="0.125", currency="EUR", cents=12)  # fails: the code warns "rounded to a cent"


def test_no_currency() -> None:
    with warns(DeprecationWarning, match="USD is assumed"):
        money = parse_amount("1", None)
    assert money.currency == "USD"


def test_below_zero() -> None:
    with raises(ValueError, match="below zero"):
        parse_amount("-1", "EUR")
