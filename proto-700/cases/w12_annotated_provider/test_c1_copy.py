from typing import Annotated

from fixtures import source, target
from sketch import From
from subject import Database, Tx, copy


def test_copy(
    src: Annotated[Tx[Database], From(source)],
    dst: Annotated[Tx[Database], From(target)],
) -> None:
    src.insert("a")
    copy(src, dst)
    assert dst.rows() == ["a"], "the target holds what the source held"
