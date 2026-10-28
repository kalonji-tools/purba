"""A `type` statement (PEP 695) for each role."""

from common import Seeded, build_database, copy, saw
from sketch import fixture
from subject import Database

POLICIES: dict[str, dict[str, bool]] = {"refuse": {}, "strict": {"strict": True}}

type SourceDb = Database
type TargetDb = Database


@fixture
def database() -> Database:
    return build_database()


@fixture
def seeded(src: SourceDb) -> Seeded:
    src.insert("ana", 1)
    return Seeded("ana")


def test_copy(src: SourceDb, dst: TargetDb, seeded: Seeded) -> None:
    saw(src, dst)
    copy(src, dst)
    assert dst.rows == src.rows
