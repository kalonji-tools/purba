from unittest import mock

import lending


def test_patch_typo_target():
    with mock.patch.object(lending, "Librarian"):
        got = lending.borrow("Dune")
    assert got == "lent Dune"
