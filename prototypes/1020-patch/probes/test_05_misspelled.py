from unittest import mock

import lending


def test_unspecced_attr():
    with mock.patch.object(lending, "Library") as fake:
        lending.borrow("Dune")
    assert fake.return_value.lendd.called == False
