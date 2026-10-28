from fixtures import SourceDb, TargetDb
from subject import Tx, copy


def test_copy(src: Tx[SourceDb], dst: Tx[TargetDb]) -> None:
    src.insert("a")
    copy(src, dst)
    assert dst.rows() == ["a"], "the target holds what the source held"
