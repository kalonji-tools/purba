"""The reagent and stock fixtures in a typed default (not from #699: a typed `Depends`)."""

from sketch import fixture, use
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


def test_copy(src: Tx[Database] = use(tx, source), dst: Tx[Database] = use(tx, target)) -> None:
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
