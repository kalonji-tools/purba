"""ex5: what the parse refuses before anything runs, and one refusal at run time."""

from app import Money, parse_amount, slugify
from form_stacked import assay, case, raises


@case("a", input=dict(text="Hi"), assay=dict(product="hi"))
def nested(text: str) -> str:
    return slugify(text.strip())


@case("a", input=dict(text="a b"), assay=dict(product="a_b"))
def literal(text: str) -> str:
    return slugify(text, "_")  # passes under --literals


@case("a", input=dict(text="Hi"), assay=dict(slgu="hi"))
def typo_key(text: str) -> str:
    return slugify(text)


@case("a", input=dict(text=" "), assay=dict(product=""), raises=raises(ValueError, "empty"))
def product_on_raise(text: str) -> str:
    return slugify(text)


@case("a", input=dict(text="Hi"), assay=dict(product="hi", echoed="Hi"))
def reads_literal(text: str) -> str:
    return slugify(text)


@assay(reads_literal)
def echoed(text: str) -> str:
    return text


@case("a", input=dict(text="12.50", currency="EUR"), assay=dict(cents=1250))
def missing_value(text: str, currency: str) -> Money:
    return parse_amount(text, currency)


@assay(missing_value)
def cents(product: Money) -> int:
    return product.cents


@assay(missing_value)
def unit(product: Money) -> str:
    return product.currency


@case("a", input=dict(text="12.50", currency="EUR"), assay=dict(product="12.50 EUR"))
def reading_not_literal(text: str, currency: str) -> Money:
    return parse_amount(text, currency)
