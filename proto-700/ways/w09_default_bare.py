"""The provider itself as the default value (Ward)."""

from sketch import fixture
from subject import Database, Tx, copy


@fixture
def source() -> Database:
    return Database("source")


@fixture
def target() -> Database:
    return Database("target")


@fixture
def tx[D: Database](db: D) -> Tx[D]:
    return Tx(db)


def test_copy(src: Tx[Database] = source, dst: Tx[Database] = target) -> None:
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
