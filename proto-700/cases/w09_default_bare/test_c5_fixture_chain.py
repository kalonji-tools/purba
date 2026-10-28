from fixtures import source, target
from sketch import fixture
from subject import Database, Seeded, Tx, copy


@fixture
def seeded(src: Tx[Database] = source) -> Seeded:
    src.insert("a")
    src.insert("b")
    return Seeded(2)


def test_copy_seeded(
    seeded: Seeded = seeded,
    src: Tx[Database] = source,
    dst: Tx[Database] = target,
) -> None:
    copy(src, dst)
    assert len(dst.rows()) == seeded.rows, "the target holds each seeded row"
