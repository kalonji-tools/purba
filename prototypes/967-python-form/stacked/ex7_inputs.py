"""ex7: a misspelled input and an input of the wrong type. Who catches each?"""

from app import slugify
from form_stacked import case


@case("plain", input=dict(text="Hello World"), assay=dict(product="hello-world"))
@case("typo",  input=dict(txt="Hello World"),  assay=dict(product="hello-world"))
@case("wrong", input=dict(text=42),            assay=dict(product="42"))
def slug_of(text: str) -> str:
    return slugify(text)


@case("wrong", input=dict(text=42), assay=dict(product="42"))
def slug_again(text: str) -> str:
    return slugify(text)
