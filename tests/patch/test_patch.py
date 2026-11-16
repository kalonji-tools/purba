"""Patches and mocks that purba refuses, and a patch that a suppression lets run."""

from unittest import mock

import lending


def test_patch_the_code_under_test():
    with mock.patch.object(lending, "borrow", return_value="lent Dune"):
        got = lending.borrow("Dune")
    assert got == "lent Dune"


class Stub:
    def lend(self, title):
        return f"stub {title}"


# purba: suppress[patched-name] the library has no seam yet
def test_borrow():
    with mock.patch.object(lending, "Library", Stub):
        got = lending.borrow("Dune")
    assert got == "stub Dune"


def test_mock():
    library = mock.Mock()
    got = library.lend("Dune")
    assert got is not None
