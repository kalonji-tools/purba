"""A role by a parameter with no annotation, found by name (rodi)."""

from sketch import fixture
from subject import Database, Tx, copy


@fixture
def source_db() -> Database:
    return Database("source")


@fixture
def target_db() -> Database:
    return Database("target")


@fixture
def source_tx(source_db: Database) -> Tx[Database]:
    return Tx(source_db)


@fixture
def target_tx(target_db: Database) -> Tx[Database]:
    return Tx(target_db)


def test_copy(source_tx, target_tx) -> None:
    copy(source_tx, target_tx)
    assert target_tx.rows() == source_tx.rows(), "the target holds what the source held"
