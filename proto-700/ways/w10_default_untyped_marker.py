"""The provider in a default marker typed Any (FastAPI's `= Depends(fn)`)."""

from sketch import fixture, use_any
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


def test_copy(src: Tx[Database] = use_any(source), dst: Tx[Database] = use_any(target)) -> None:
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
