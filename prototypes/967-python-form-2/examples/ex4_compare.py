"""ex4: how a reading is compared: a subclass of str is a str, True is not 1, 1.0 is not 1."""

from app import adult_count
from experiment import experiment


class Message(str):
    """Like loguru's: a str with more on it."""


@experiment
def message_of(text: str) -> list[str]:
    return [Message(text)]


@experiment
def adults(ages: list[int]) -> int:
    return adult_count(ages)


@experiment
def halved(value: int) -> float:
    return value / 2


subclass = message_of(text="a\n").expect(product=["a\n"])
two      = adults(ages=[20, 30]).expect(product=2)
one      = adults(ages=[20]).expect(product=1)  # adult_count returns True for one age: a bug this case finds
half     = halved(value=3).expect(product=1.5)
whole    = halved(value=4).expect(product=2.0)
