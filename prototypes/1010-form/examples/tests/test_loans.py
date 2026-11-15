"""Lending: a pipeline of stages, from a member of test_members.py to a book given back late. Each
stage checks its own step, and later tests start from it."""

from __future__ import annotations

from cases import before, case, raises, use
from lending import Closed, Loan, Member, NotOnShelf, clear_notices, notices
from test_members import ann, bob


def test_borrow(title: str, due: int, member: Member = use(ann)) -> Loan:
    """A stage on a stage of another module: ann borrows a book."""
    loan = member.library.lend(member, title)
    assert loan.due == due
    assert member.library.shelf == ["Emma"]
    return loan


dune = case(test_borrow, title="Dune", due=21)


def test_late(days: int, loan: Loan = use(dune)) -> Loan:
    """A stage on `dune`: time passes."""
    loan.member.library.wait(days)
    assert loan.member.library.today == days
    return loan


late = case(test_late, days=30)


@before(clear_notices)
def test_give_back(fine: int, notice: str, loan: Loan = use(late)) -> None:
    """Three stages up: the book comes back nine days late. `@before` drops the notice of the loan,
    which an earlier stage sent."""
    paid = loan.member.library.give_back(loan)
    assert paid == fine
    assert loan.member.library.shelf == ["Emma", "Dune"]
    assert notices() == [notice]


nine_days = case(test_give_back, fine=90, notice="ann owes 90")


def test_lent_book_is_not_on_the_shelf(loan: Loan = use(dune)) -> None:
    """An expected raise: Dune is out."""
    with raises(NotOnShelf, match="Dune") as caught:
        loan.member.library.lend(loan.member, "Dune")
    assert str(caught.value) == "'Dune' is not on the shelf"


def test_closed_library_lends_nothing(member: Member = use(bob)) -> None:
    """A `with` setup: while the library is closed, a loan raises; after it, the shelf is whole."""
    library = member.library
    with library.closed():
        with raises(Closed):
            library.lend(member, "Emma")
    assert library.open is True
    assert library.shelf == ["Dune", "Emma"]
