from unittest import mock

import lending


def test_with_patch_object():
    with mock.patch.object(lending, "Library") as fake:
        fake.return_value.lend.return_value = "lent X"
        got = lending.borrow("Dune")
    assert got == "lent X"
