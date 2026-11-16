from unittest import mock

import lending


def test_call_count():
    with mock.patch.object(lending, "Library") as fake:
        lending.borrow("Dune")
    assert fake.return_value.lend.call_count == 1
