"""A role as an Annotated marker string (injector, svcs)."""

from typing import Annotated

from sketch import fixture
from subject import Database, Tx, copy


@fixture
def source() -> Annotated[Database, "source"]:
    return Database("source")


@fixture
def target() -> Annotated[Database, "target"]:
    return Database("target")


@fixture
def tx[D: Database](db: D) -> Tx[D]:
    return Tx(db)


def test_copy(src: Annotated[Tx[Database], "source"], dst: Annotated[Tx[Database], "target"]) -> None:
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
