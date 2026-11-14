"""FORM.md: the runner refuses "an input of the wrong type", checking each input's type
"when it can read the annotation at run time". Here each annotation is a plain runtime class.

Expect: both refused before the run.
"""

from pathlib import Path

from cases import case, use


def text() -> str:
    return "not a path"


def test_default_of_wrong_type(n: int = "3"):  # type: ignore  # the seeded fault
    got = n * 2
    assert got == 6


def test_fixture_of_wrong_type(p: Path = use(text)):  # type: ignore  # the seeded fault
    got = str(p)
    assert got == "not a path"


def test_case_of_wrong_type(n: int):
    got = n * 2
    assert got == 6


given_wrong = case(test_case_of_wrong_type, n="3")  # type: ignore  # the control: refused as FORM says
