"""ex7: a misspelled input and an input of the wrong type. Who catches each?"""

from app import slugify
from form_klasscall import Experiment, case


class SlugOf(Experiment):
    def act(text: str) -> str:
        return slugify(text)

    plain = case(act, text="Hello World").expect(product="hello-world")
    typo  = case(act, txt="Hello World").expect(product="hello-world")
    wrong = case(act, text=42).expect(product="42")


class SlugAgain(Experiment):
    def act(text: str) -> str:
        return slugify(text)

    wrong = case(act, text=42).expect(product="42")
