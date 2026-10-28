from fixtures import SourceDb, TargetDb
from sketch import fixture
from subject import Seeded, Tx, copy


@fixture
def seeded(src: Tx[SourceDb]) -> Seeded:
    src.insert("a")
    src.insert("b")
    return Seeded(2)


def test_copy_seeded(seeded: Seeded, src: Tx[SourceDb], dst: Tx[TargetDb]) -> None:
    copy(src, dst)
    assert len(dst.rows()) == seeded.rows, "the target holds each seeded row"
