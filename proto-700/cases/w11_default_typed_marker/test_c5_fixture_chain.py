from fixtures import source, target, tx
from sketch import fixture, use
from subject import Database, Seeded, Tx, copy


@fixture
def seeded(src: Tx[Database] = use(tx, source)) -> Seeded:
    src.insert("a")
    src.insert("b")
    return Seeded(2)


def test_copy_seeded(
    seeded: Seeded = use(seeded),
    src: Tx[Database] = use(tx, source),
    dst: Tx[Database] = use(tx, target),
) -> None:
    copy(src, dst)
    assert len(dst.rows()) == seeded.rows, "the target holds each seeded row"
