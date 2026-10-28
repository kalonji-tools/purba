"""An alias that hides the provider marker, as FastAPI's docs advise."""

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


SourceTx = Annotated[Tx[Database], From(source)]
TargetTx = Annotated[Tx[Database], From(target)]


def test_copy(src: SourceTx, dst: TargetTx) -> None:
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
