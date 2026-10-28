from fixtures import source, target
from subject import Database, Tx, copy


def test_copy(src: Tx[Database] = source, dst: Tx[Database] = target) -> None:
    src.insert("a")
    copy(src, dst)
    assert dst.rows() == ["a"], "the target holds what the source held"
