from fixtures import SourceTx, TargetTx
from subject import copy


def test_copy(src: SourceTx, dst: TargetTx) -> None:
    src.insert("a")
    copy(src, dst)
    assert dst.rows() == ["a"], "the target holds what the source held"
