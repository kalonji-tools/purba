"""ex1: a pure function. The built-in assay `product` is what the act returns."""

from app import slugify
from experiment import experiment


@experiment
def slug_of(text: str, sep: str = "-") -> str:
    return slugify(text, sep)


plain   = slug_of(text="Hello World").expect(product="hello-world")
under   = slug_of(text="Hello World", sep="_").expect(product="hello_world")
accents = slug_of(text="Crème Brûlée").expect(product="creme-brulee")  # app.slugify keeps accents: a bug this case finds
empty   = slug_of(text="   ").raises(ValueError, match="^empty$")
