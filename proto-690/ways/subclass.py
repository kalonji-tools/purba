"""A subclass for each role. No runtime unwrap: each role needs a builder."""

from common import Seeded, build_database, copy, saw
from sketch import fixture
from subject import Database

POLICIES: dict[str, dict[str, bool]] = {"refuse": {}, "strict": {"strict": True}}


class SourceDb(Database):
    pass


class TargetDb(Database):
    pass


@fixture
def database() -> Database:
    return build_database()


@fixture
def source() -> SourceDb:
    return SourceDb()  # cannot reuse build_database: it returns a Database


@fixture
def target() -> TargetDb:
    return TargetDb()


@fixture
def seeded(src: SourceDb) -> Seeded:
    src.insert("ana", 1)
    return Seeded("ana")


def test_copy(src: SourceDb, dst: TargetDb, seeded: Seeded) -> None:
    saw(src, dst)
    copy(src, dst)
    assert dst.rows == src.rows
