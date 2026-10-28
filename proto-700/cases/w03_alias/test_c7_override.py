from fixtures import SourceDb, SourceTx, TargetTx
from sketch import fixture
from subject import Database, copy


@fixture
def source() -> SourceDb:
    big = Database("big source")
    big.table.extend(["a", "b", "c"])
    return SourceDb(big)


def test_copy_all(src: SourceTx, dst: TargetTx) -> None:
    copy(src, dst)
    assert dst.rows() == ["a", "b", "c"], "the target holds every row"


def test_source_is_big(src: SourceTx) -> None:
    assert len(src.rows()) == 3, "the source holds three rows"


def test_copy_twice(src: SourceTx, dst: TargetTx) -> None:
    copy(src, dst)
    copy(src, dst)
    assert len(dst.rows()) == 6, "each copy appends every row"
