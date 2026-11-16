"""A blocked stage, on purpose: `two_weeks` expects a loan of 14 days, and loans last 21. The stage
fails, and each later stage is blocked, with a line that names it."""

from __future__ import annotations

from cases import case, use
from lending import Library, Loan


def test_borrow(due: int, library: Library = use(Library)) -> Loan:
    library.stock("Dune")
    loan = library.lend(library.join("ann"), "Dune")
    assert loan.due == due
    return loan


two_weeks = case(test_borrow, due=14)


def test_on_time(loan: Loan = use(two_weeks)) -> None:
    loan.member.library.wait(14)
    fine = loan.member.library.give_back(loan)
    assert fine == 0
