"""A parameter name mapped to a provider in a decorator (Ward's @using, pytest-lazy-fixtures' lf)."""

from sketch import fixture, using
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


@using(src=source, dst=target)
def test_copy(src: Tx[Database], dst: Tx[Database]) -> None:
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
