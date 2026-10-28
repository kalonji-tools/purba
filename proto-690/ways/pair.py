"""The test names a tuple, and fixtures builds each member afresh."""

from common import Seeded, build_database, copy, saw
from sketch import fixture
from subject import Database

POLICIES: dict[str, dict[str, bool]] = {"refuse": {}, "strict": {"strict": True}}


@fixture
def database() -> Database:
    return build_database()


@fixture
def seeded(dbs: tuple[Database, Database]) -> Seeded:
    dbs[0].insert("ana", 1)  # which member is the source is a convention
    return Seeded("ana")


def test_copy(dbs: tuple[Database, Database], seeded: Seeded) -> None:
    src, dst = dbs
    saw(src, dst)
    copy(src, dst)
    assert dst.rows == src.rows
