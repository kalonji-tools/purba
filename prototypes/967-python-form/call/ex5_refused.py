"""ex5: what the parse refuses before anything runs, and one refusal at run time."""

from app import Money, parse_amount, slugify
from form_call import experiment


@experiment
def nested(text: str) -> str:
    return slugify(text.strip())


a1 = nested(text="Hi").expect(product="hi")


@experiment
def literal(text: str) -> str:
    return slugify(text, "_")  # passes under --literals


a2 = literal(text="a b").expect(product="a_b")


@experiment
def typo_key(text: str) -> str:
    return slugify(text)


a3 = typo_key(text="Hi").expect(slgu="hi")


@experiment
def product_on_raise(text: str) -> str:
    return slugify(text)


a4 = product_on_raise(text=" ").raises(ValueError, "empty").expect(product="")


@experiment
def reads_literal(text: str) -> str:
    return slugify(text)


@reads_literal.assay
def echoed(text: str) -> str:
    return text


a5 = reads_literal(text="Hi").expect(product="hi", echoed="Hi")


@experiment
def missing_value(text: str, currency: str) -> Money:
    return parse_amount(text, currency)


@missing_value.assay
def cents(product: Money) -> int:
    return product.cents


@missing_value.assay
def unit(product: Money) -> str:
    return product.currency


a6 = missing_value(text="12.50", currency="EUR").expect(cents=1250)


@experiment
def reading_not_literal(text: str, currency: str) -> Money:
    return parse_amount(text, currency)


a7 = reading_not_literal(text="12.50", currency="EUR").expect(product="12.50 EUR")


@experiment
def unnamed(text: str) -> str:
    return slugify(text)


unnamed(text="Hi").expect(product="hi")
