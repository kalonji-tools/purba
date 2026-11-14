"""ex7: a misspelled input and an input of the wrong type. Who catches each?"""

from app import slugify
from form_call import experiment


@experiment
def slug_of(text: str) -> str:
    return slugify(text)


plain = slug_of(text="Hello World").expect(product="hello-world")
typo  = slug_of(txt="Hello World").expect(product="hello-world")
wrong = slug_of(text=42).expect(product="42")


@experiment
def slug_again(text: str) -> str:
    return slugify(text)


wrong_again = slug_again(text=42).expect(product="42")
