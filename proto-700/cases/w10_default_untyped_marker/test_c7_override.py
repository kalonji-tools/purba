from fixtures import big_source, target
from sketch import use_any
from subject import Database, Tx, copy


def test_copy_all(
    src: Tx[Database] = use_any(big_source),
    dst: Tx[Database] = use_any(target),
) -> None:
    copy(src, dst)
    assert dst.rows() == ["a", "b", "c"], "the target holds every row"


def test_source_is_big(src: Tx[Database] = use_any(big_source)) -> None:
    assert len(src.rows()) == 3, "the source holds three rows"


def test_copy_twice(
    src: Tx[Database] = use_any(big_source),
    dst: Tx[Database] = use_any(target),
) -> None:
    copy(src, dst)
    copy(src, dst)
    assert len(dst.rows()) == 6, "each copy appends every row"
