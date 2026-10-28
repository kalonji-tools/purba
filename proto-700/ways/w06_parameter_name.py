"""A role by parameter name (pytest, Litestar). The annotation is not read."""

from sketch import fixture
from subject import Database, Tx, copy


@fixture
def source_db() -> Database:
    return Database("source")


@fixture
def target_db() -> Database:
    return Database("target")


@fixture
def source(source_db: Database) -> Tx[Database]:
    return Tx(source_db)


@fixture
def target(target_db: Database) -> Tx[Database]:
    return Tx(target_db)


def test_copy(source: Tx[Database], target: Tx[Database]) -> None:
    copy(source, target)
    assert target.rows() == source.rows(), "the target holds what the source held"
