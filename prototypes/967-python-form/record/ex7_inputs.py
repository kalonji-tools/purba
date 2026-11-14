"""ex7: a misspelled input and an input of the wrong type. Who catches each?"""

from app import slugify
from form_record import Experiment


class SlugOf(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)


plain = SlugOf(text="Hello World").expect(product="hello-world")
typo  = SlugOf(txt="Hello World").expect(product="hello-world")
wrong = SlugOf(text=42).expect(product="42")


class SlugAgain(Experiment[str]):
    text: str

    def act(self) -> str:
        return slugify(self.text)


wrong_again = SlugAgain(text=42).expect(product="42")
