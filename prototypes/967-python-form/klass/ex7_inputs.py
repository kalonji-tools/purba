"""ex7: a misspelled input and an input of the wrong type. Who catches each?"""

from app import slugify
from form_klass import Experiment, case


class SlugOf(Experiment):
    def act(text: str) -> str:
        return slugify(text)

    plain = case(text="Hello World", product="hello-world")
    typo  = case(txt="Hello World",  product="hello-world")
    wrong = case(text=42,            product="42")


class SlugAgain(Experiment):
    def act(text: str) -> str:
        return slugify(text)

    wrong = case(text=42, product="42")
