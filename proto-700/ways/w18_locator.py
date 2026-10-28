"""A role asked of a locator in the body (svcs). The signature names the container."""

from typing import NewType

from sketch import Fixtures, fixture
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


def test_copy(fx: Fixtures) -> None:
    src, dst = fx.get(Tx[SourceDb]), fx.get(Tx[TargetDb])
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
