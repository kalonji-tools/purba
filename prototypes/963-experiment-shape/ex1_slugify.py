"""ex1: a pure function. The easy case: one table, one act, one assay."""

from experiment import experiment, raises
from prototype_app import slugify


@experiment(
    columns=("text",              "slug",          "raises"),
    plain=  ("Hello World",       "hello-world",   ...),
    spaces= ("  many   spaces ",  "many-spaces",   ...),
    accents=("Crème Brûlée",      "creme-brulee",  ...),  # fails on purpose: shows a failure
    empty=  ("   ",               ...,             raises(ValueError, "empty")),
)
def slug_of(text: str) -> str:
    return slugify(text)


@slug_of.assay
def slug(product: str) -> str:
    return product
