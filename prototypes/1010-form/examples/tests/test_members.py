"""Joining the library. Fixtures build the library and a member; a stage hands a member to later
tests."""

from __future__ import annotations

from cases import case, use
from lending import Library, Member


def stocked() -> Library:
    """A fixture: a library with two books on its shelf."""
    library = Library()
    library.stock("Dune", "Emma")
    return library


def cat(library: Library = use(stocked)) -> Member:
    """A fixture that uses a fixture: cat, a member of the stocked library."""
    return library.join("cat")


def test_lend_to_a_member(member: Member = use(cat), library: Library = use(stocked)) -> None:
    """The test and `cat` share one stocked library in each case."""
    library.lend(member, "Dune")
    assert library.shelf == ["Emma"]
    assert [loan.member.name for loan in library.loans] == ["cat"]


def test_join(name: str, library: Library = use(stocked)) -> Member:
    """A stage: it checks its own act, then hands the member on."""
    member = library.join(name)
    assert library.members == [name]
    assert member.name == name
    return member


ann = case(test_join, name="ann")
bob = case(test_join, name="bob")


def test_second_member(member: Member = use(ann)) -> None:
    """Takes the stage `ann`: bob joins the same library after her."""
    member.library.join("bob")
    assert member.library.members == ["ann", "bob"]
