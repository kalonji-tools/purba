from typing import Annotated

from fixtures import source, target
from sketch import From, fixture
from subject import Database, Seeded, Tx, copy


@fixture
def seeded(src: Annotated[Tx[Database], From(source)]) -> Seeded:
    src.insert("a")
    src.insert("b")
    return Seeded(2)


def test_copy_seeded(
    seeded: Annotated[Seeded, From(seeded)],
    src: Annotated[Tx[Database], From(source)],
    dst: Annotated[Tx[Database], From(target)],
) -> None:
    copy(src, dst)
    assert len(dst.rows()) == seeded.rows, "the target holds each seeded row"
