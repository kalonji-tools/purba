"""FORM.md: "A function that the test calls itself, under any name, is part of the act, and has one path too."

Expect each test refused: each calls a function with an `if` (and one with an assert).
"""

from helper_lab import check_dir_like, two_paths


def local_two_paths(n: int) -> int:
    if n > 0:
        return n
    return -n


def test_imported_helper_with_if_and_assert():
    names = ["a"]
    check_dir_like(names, size=2)
    assert names == ["a"]


def test_imported_helper_with_if():
    got = two_paths(-3)
    assert got == 3


def test_local_helper_with_if():
    got = local_two_paths(-3)
    assert got == 3
