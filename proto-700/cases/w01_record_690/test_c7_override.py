from fixtures import SourceDb, TargetDb
from sketch import fixture
from subject import Database, Tx, copy


@fixture
def source() -> SourceDb:
    big = Database("big source")
    big.table.extend(["a", "b", "c"])
    return SourceDb(big)


def test_copy_all(src: Tx[SourceDb], dst: Tx[TargetDb]) -> None:
    copy(src, dst)
    assert dst.rows() == ["a", "b", "c"], "the target holds every row"


def test_source_is_big(src: Tx[SourceDb]) -> None:
    assert len(src.rows()) == 3, "the source holds three rows"


def test_copy_twice(src: Tx[SourceDb], dst: Tx[TargetDb]) -> None:
    copy(src, dst)
    copy(src, dst)
    assert len(dst.rows()) == 6, "each copy appends every row"
