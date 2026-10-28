from sketch import fixture
from subject import Database, Seeded, Tx, copy


@fixture
def seeded(src: Tx[Database]) -> Seeded:
    src.insert("a")
    src.insert("b")
    return Seeded(2)


def test_copy_seeded(seeded: Seeded, src: Tx[Database], dst: Tx[Database]) -> None:
    copy(src, dst)
    assert len(dst.rows()) == seeded.rows, "the target holds each seeded row"
