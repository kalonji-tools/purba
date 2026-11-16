from unittest import mock

import lending


def test_spec_mismatch_check_reads_mock():
    with mock.patch.object(lending, "Library", autospec=True) as fake:
        lending.borrow("Dune")
        assert fake.return_value.lend.call_count == 1
