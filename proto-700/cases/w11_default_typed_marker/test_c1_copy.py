from fixtures import source, target, tx
from sketch import use
from subject import Database, Tx, copy


def test_copy(src: Tx[Database] = use(tx, source), dst: Tx[Database] = use(tx, target)) -> None:
    src.insert("a")
    copy(src, dst)
    assert dst.rows() == ["a"], "the target holds what the source held"
