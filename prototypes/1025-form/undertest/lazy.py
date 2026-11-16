"""The code under test for #1025: lazy products, a generator that raises, a parser that raises."""


def gen():
    yield 1
    raise ValueError("boom")


def parse(text: str) -> int:
    if not text.isdigit():
        raise ValueError(f"not a number: {text!r}")
    return int(text)


def numbers(text: str):
    """A generator: one number for each word of the text."""
    for word in text.split():
        yield int(word)


def chunks(text: str, size: int) -> list[str]:
    return [text[i : i + size] for i in range(0, len(text), size)]


class Word:
    def __init__(self, text: str, category: object) -> None:
        self.text, self.category = text, category

    def __repr__(self) -> str:
        return f"Word({self.text!r})"


def tag(text: str) -> list[Word]:
    return [Word(w, None if w.isdigit() else str) for w in text.split()]
