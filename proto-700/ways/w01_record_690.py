"""The syntax of the #690 record: a role on the stock, one generic reagent."""

from typing import NewType

from sketch import fixture
from subject import Database, Tx, copy

SourceDb = NewType("SourceDb", Database)
TargetDb = NewType("TargetDb", Database)


@fixture
def source() -> SourceDb:
    return SourceDb(Database("source"))


@fixture
def target() -> TargetDb:
    return TargetDb(Database("target"))


@fixture
def tx[D: Database](db: D) -> Tx[D]:
    return Tx(db)


def test_copy(src: Tx[SourceDb], dst: Tx[TargetDb]) -> None:
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
