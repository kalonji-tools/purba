from fixtures import source, target
from sketch import fixture, using
from subject import Database, Seeded, Tx, copy


@fixture
@using(src=source)
def seeded(src: Tx[Database]) -> Seeded:
    src.insert("a")
    src.insert("b")
    return Seeded(2)


@using(seeded=seeded, src=source, dst=target)
def test_copy_seeded(seeded: Seeded, src: Tx[Database], dst: Tx[Database]) -> None:
    copy(src, dst)
    assert len(dst.rows()) == seeded.rows, "the target holds each seeded row"
