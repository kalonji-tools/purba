"""ex9: cases that a loop makes, and a family of cases through a subclass. What does purba see?"""

from app import slugify
from form_record_strict import Experiment


class SlugOf(Experiment[str]):
    text: str
    sep: str = "-"

    def act(self) -> str:
        return slugify(self.text, self.sep)


plain = SlugOf(text="a b").expect(product="a-b")
grid = [SlugOf(text=t, sep=s).expect(product=t.replace(" ", s)) for t in ("a b", "c d") for s in "-_"]


class SlugUnderscore(SlugOf):  # a family: every case of it shares sep="_"
    sep: str = "_"


under = SlugUnderscore(text="a b").expect(product="a_b")
