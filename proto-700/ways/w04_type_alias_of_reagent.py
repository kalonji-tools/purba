"""As w03, with `type SourceTx = Tx[SourceDb]`: a new key at runtime."""

from typing import NewType

from sketch import fixture
from subject import Database, Tx, copy

SourceDb = NewType("SourceDb", Database)
TargetDb = NewType("TargetDb", Database)
type SourceTx = Tx[SourceDb]
type TargetTx = Tx[TargetDb]


@fixture
def source() -> SourceDb:
    return SourceDb(Database("source"))


@fixture
def target() -> TargetDb:
    return TargetDb(Database("target"))


@fixture
def tx[D: Database](db: D) -> Tx[D]:
    return Tx(db)


def test_copy(src: SourceTx, dst: TargetTx) -> None:
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
