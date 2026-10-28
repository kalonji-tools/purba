"""A role by tuple position (pytest-cases' unpack_fixture): one fixture builds both."""

from sketch import fixture
from subject import Database, Tx, copy

CopyPair = tuple[Tx[Database], Tx[Database]]


@fixture
def pair() -> CopyPair:
    return Tx(Database("source")), Tx(Database("target"))


def test_copy(pair: CopyPair) -> None:
    src, dst = pair
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
