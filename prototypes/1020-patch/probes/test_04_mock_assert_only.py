from unittest import mock

import lending


def test_only_mock_assert():
    with mock.patch.object(lending, "Library") as fake:
        lending.borrow("Dune")
    fake.return_value.lend.assert_called_once_with("Dune")
