"""ex7: what a type checker sees, before the run. Each line below is written to be caught."""

from app import slugify
from experiment import experiment


@experiment
def slug_of(text: str, sep: str = "-") -> str:
    return slugify(text, sep)


misspelled = slug_of(txt="Hello").expect(product="hello")
wrong_type = slug_of(text=42).expect(product="42")
wrong_prod = slug_of(text="Hello").expect(product=5)
