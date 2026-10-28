"""The provider as a container attribute (dependency-injector's `Provide[Container.x]`)."""

from sketch import Provide, fixture
from subject import Database, Tx, copy


class Stocks:
    @staticmethod
    @fixture
    def source() -> Database:
        return Database("source")

    @staticmethod
    @fixture
    def target() -> Database:
        return Database("target")


@fixture
def tx[D: Database](db: D) -> Tx[D]:
    return Tx(db)


def test_copy(src: Tx[Database] = Provide[Stocks.source], dst: Tx[Database] = Provide[Stocks.target]) -> None:
    copy(src, dst)
    assert dst.rows() == src.rows(), "the target holds what the source held"
