"""As #690, with each parameter named after its role (a control, not from #699)."""

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


def test_copy(source: Tx[SourceDb], target: Tx[TargetDb]) -> None:
    copy(source, target)
    assert target.rows() == source.rows(), "the target holds what the source held"
