from fixtures import SourceTx, TargetTx
from sketch import fixture
from subject import Seeded, copy


@fixture
def seeded(src: SourceTx) -> Seeded:
    src.insert("a")
    src.insert("b")
    return Seeded(2)


def test_copy_seeded(seeded: Seeded, src: SourceTx, dst: TargetTx) -> None:
    copy(src, dst)
    assert len(dst.rows()) == seeded.rows, "the target holds each seeded row"
