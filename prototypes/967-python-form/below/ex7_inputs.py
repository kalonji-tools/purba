"""ex7: a misspelled input and an input of the wrong type. Who catches each?"""

from app import slugify
from form_below import experiment


@experiment
def slug_of(text: str) -> str:
    return slugify(text)


slug_of.case("plain", text="Hello World").expect(product="hello-world")
slug_of.case("typo", txt="Hello World").expect(product="hello-world")
slug_of.case("wrong", text=42).expect(product="42")


@experiment
def slug_again(text: str) -> str:
    return slugify(text)


slug_again.case("wrong", text=42).expect(product="42")
