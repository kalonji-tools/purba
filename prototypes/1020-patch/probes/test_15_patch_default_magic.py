from unittest import mock

import lending


def test_patch_default_value_answers_anything():
    with mock.patch.object(lending, "Library"):
        got = lending.borrow("Dune")
    assert type(got).__name__ == "MagicMock"
