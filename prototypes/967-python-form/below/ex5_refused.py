"""ex5: what the parse refuses before anything runs, and one refusal at run time."""

from app import Money, parse_amount, slugify
from form_below import experiment


@experiment
def nested(text: str) -> str:
    return slugify(text.strip())


nested.case("a", text="Hi").expect(product="hi")


@experiment
def literal(text: str) -> str:
    return slugify(text, "_")  # passes under --literals


literal.case("a", text="a b").expect(product="a_b")


@experiment
def typo_key(text: str) -> str:
    return slugify(text)


typo_key.case("a", text="Hi").expect(slgu="hi")


@experiment
def product_on_raise(text: str) -> str:
    return slugify(text)


product_on_raise.case("a", text=" ").raises(ValueError, "empty").expect(product="")


@experiment
def reads_literal(text: str) -> str:
    return slugify(text)


@reads_literal.assay
def echoed(text: str) -> str:
    return text


reads_literal.case("a", text="Hi").expect(product="hi", echoed="Hi")


@experiment
def missing_value(text: str, currency: str) -> Money:
    return parse_amount(text, currency)


@missing_value.assay
def cents(product: Money) -> int:
    return product.cents


@missing_value.assay
def unit(product: Money) -> str:
    return product.currency


missing_value.case("a", text="12.50", currency="EUR").expect(cents=1250)


@experiment
def reading_not_literal(text: str, currency: str) -> Money:
    return parse_amount(text, currency)


reading_not_literal.case("a", text="12.50", currency="EUR").expect(product="12.50 EUR")
