"""Joining the library. A fixture builds the library; a stage hands a member to later tests."""

from __future__ import annotations

from cases import case, use
from lending import Library, Member


def stocked() -> Library:
    """A fixture: a library with two books on its shelf."""
    library = Library()
    library.stock("Dune", "Emma")
    return library


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
