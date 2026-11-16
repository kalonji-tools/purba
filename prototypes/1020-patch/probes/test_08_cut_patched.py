from unittest import mock

import lending


def test_patch_the_code_under_test():
    with mock.patch.object(lending, "borrow", return_value="lent Dune"):
        got = lending.borrow("Dune")
    assert got == "lent Dune"
