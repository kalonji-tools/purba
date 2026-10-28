"""The provider in an Annotated marker (FastAPI's `Annotated[T, Depends(fn)]`)."""

from typing import Annotated

from sketch import From, fixture
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


def test_copy(src: Annotated[Tx[Database], From(source)], dst: Annotated[Tx[Database], From(target)]) -> None:
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
