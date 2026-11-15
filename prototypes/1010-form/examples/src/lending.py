"""The code under test of the examples: a small lending library."""

from __future__ import annotations

from collections.abc import Iterator
from contextlib import contextmanager
from dataclasses import dataclass, field

LOAN_DAYS = 21
FINE_PER_DAY = 10
SENT: list[str] = []  # every notice the library sent: global state, as a logger keeps its sinks


def notices() -> list[str]:
    return list(SENT)


def clear_notices() -> None:
    SENT.clear()


class Closed(Exception):
    pass


class NotOnShelf(Exception):
    pass


@dataclass
class Member:
    name: str
    library: Library


@dataclass
class Loan:
    member: Member
    title: str
    due: int
    returned: int | None = None


@dataclass
class Library:
    shelf: list[str] = field(default_factory=list)
    members: list[str] = field(default_factory=list)
    loans: list[Loan] = field(default_factory=list)
    today: int = 0
    open: bool = True

    def stock(self, *titles: str) -> None:
        self.shelf.extend(titles)

    def join(self, name: str) -> Member:
        self.members.append(name)
        return Member(name, self)

    def lend(self, member: Member, title: str) -> Loan:
        if not self.open:
            raise Closed("the library is closed")
        if title not in self.shelf:
            raise NotOnShelf(f"{title!r} is not on the shelf")
        self.shelf.remove(title)
        loan = Loan(member, title, due=self.today + LOAN_DAYS)
        self.loans.append(loan)
        SENT.append(f"{member.name} borrowed {title}")
        return loan

    def give_back(self, loan: Loan) -> int:
        """Put the book back, and return the fine."""
        loan.returned = self.today
        self.shelf.append(loan.title)
        fine = max(0, self.today - loan.due) * FINE_PER_DAY
        SENT.append(f"{loan.member.name} owes {fine}")
        return fine

    def wait(self, days: int) -> None:
        self.today += days

    @contextmanager
    def closed(self) -> Iterator[None]:
        self.open = False
        try:
            yield
        finally:
            self.open = True
