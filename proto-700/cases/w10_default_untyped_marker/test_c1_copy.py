from fixtures import source, target
from sketch import use_any
from subject import Database, Tx, copy


def test_copy(src: Tx[Database] = use_any(source), dst: Tx[Database] = use_any(target)) -> None:
    src.insert("a")
    copy(src, dst)
    assert dst.rows() == ["a"], "the target holds what the source held"
