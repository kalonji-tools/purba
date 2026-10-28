from fixtures import big_source, target
from sketch import using
from subject import Database, Tx, copy


@using(src=big_source, dst=target)
def test_copy_all(src: Tx[Database], dst: Tx[Database]) -> None:
    copy(src, dst)
    assert dst.rows() == ["a", "b", "c"], "the target holds every row"


@using(src=big_source)
def test_source_is_big(src: Tx[Database]) -> None:
    assert len(src.rows()) == 3, "the source holds three rows"


@using(src=big_source, dst=target)
def test_copy_twice(src: Tx[Database], dst: Tx[Database]) -> None:
    copy(src, dst)
    copy(src, dst)
    assert len(dst.rows()) == 6, "each copy appends every row"
