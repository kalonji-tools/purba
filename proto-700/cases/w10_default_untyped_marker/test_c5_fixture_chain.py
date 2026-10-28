from fixtures import source, target
from sketch import fixture, use_any
from subject import Database, Seeded, Tx, copy


@fixture
def seeded(src: Tx[Database] = use_any(source)) -> Seeded:
    src.insert("a")
    src.insert("b")
    return Seeded(2)


def test_copy_seeded(
    seeded: Seeded = use_any(seeded),
    src: Tx[Database] = use_any(source),
    dst: Tx[Database] = use_any(target),
) -> None:
    copy(src, dst)
    assert len(dst.rows()) == seeded.rows, "the target holds each seeded row"
