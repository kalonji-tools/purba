"""A role as a marker object (dishka's FromComponent, wireup's qualifier)."""

from typing import Annotated

from sketch import Role, fixture
from subject import Database, Tx, copy


@fixture
def source() -> Annotated[Database, Role("source")]:
    return Database("source")


@fixture
def target() -> Annotated[Database, Role("target")]:
    return Database("target")


@fixture
def tx[D: Database](db: D) -> Tx[D]:
    return Tx(db)


def test_copy(src: Annotated[Tx[Database], Role("source")], dst: Annotated[Tx[Database], Role("target")]) -> None:
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
