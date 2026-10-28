"""A role by NewType of the reagent (injector, lagom, rodi, punq). A reagent fixture per role."""

from typing import NewType

from sketch import fixture
from subject import Database, Tx, copy

SourceDb = NewType("SourceDb", Database)
TargetDb = NewType("TargetDb", Database)
SourceTx = NewType("SourceTx", Tx[Database])
TargetTx = NewType("TargetTx", Tx[Database])


@fixture
def source() -> SourceDb:
    return SourceDb(Database("source"))


@fixture
def target() -> TargetDb:
    return TargetDb(Database("target"))


@fixture
def source_tx(db: SourceDb) -> SourceTx:
    return SourceTx(Tx(db))


@fixture
def target_tx(db: TargetDb) -> TargetTx:
    return TargetTx(Tx(db))


def test_copy(src: SourceTx, dst: TargetTx) -> None:
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
