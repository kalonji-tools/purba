from unittest import mock

import lending


@mock.patch.object(lending, "Library")
def test_patch_decorator(fake):
    fake.return_value.lend.return_value = "x"
    got = lending.borrow("Dune")
    assert got == "x"
