"""ex10: the fixture of a stock declares each offer as a base. What purba refuses before the run."""

from typing import Self

from engine import Samples, Stock
from form_record_strict import UNREAD, Experiment, use


class Shelf:
    def __init__(self) -> None:
        self.books: list[str] = []


class Misspelled(Stock[Shelf], Samples[Shelf]):
    def open(self) -> Shelf:
        return Shelf()

    def smaple(self, value: Shelf) -> Shelf:  # a typo: the class stays abstract
        return value

    def discard(self, value: Shelf) -> None: ...


class NoOffer(Stock[Shelf]):
    def open(self) -> Shelf:
        return Shelf()


class SampleOnly(Stock[Shelf], Samples[Shelf]):
    def open(self) -> Shelf:
        return Shelf()

    def sample(self, value: Shelf) -> Shelf:
        return Shelf()

    def discard(self, value: Shelf) -> None: ...


class Shelved(Experiment[None]):
    title: str
    shelf: Shelf = use(Misspelled)

    def act(self) -> None:
        return self.shelf.books.append(self.title)

    def count(self) -> int:
        return len(self.shelf.books)

    def expect(self, *, count: int) -> Self:
        return self


one = Shelved(title="Dune").expect(count=1)


class Bare(Experiment[None]):
    title: str
    shelf: Shelf = use(NoOffer)  # ty: no overload of `use` matches

    def act(self) -> None:
        return self.shelf.books.append(self.title)

    def count(self) -> int:
        return len(self.shelf.books)

    def expect(self, *, count: int) -> Self:
        return self


two = Bare(title="Dune").expect(count=1)


class TwoReaders(Experiment[None]):
    title: str
    shelf: Shelf = use(SampleOnly)

    def act(self) -> None:
        return self.shelf.books.append(self.title)

    def count(self) -> int:
        return len(self.shelf.books)

    def titles(self) -> list[str]:
        return self.shelf.books

    def expect(self, *, count: int, titles: list[str]) -> Self:
        return self


three = TwoReaders(title="Dune").expect(count=1, titles=["Dune"])
