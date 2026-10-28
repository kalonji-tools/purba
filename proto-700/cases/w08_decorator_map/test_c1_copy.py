from fixtures import source, target
from sketch import using
from subject import Database, Tx, copy


@using(src=source, dst=target)
def test_copy(src: Tx[Database], dst: Tx[Database]) -> None:
    src.insert("a")
    copy(src, dst)
    assert dst.rows() == ["a"], "the target holds what the source held"
