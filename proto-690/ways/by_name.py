"""Two inputs of one type, told apart by parameter name, as pytest does."""

from common import Seeded, build_database, copy, saw
from sketch import fixture
from subject import Database

POLICIES = {"by name": {"by_name": True}, "by name, strict": {"by_name": True, "strict": True}}


@fixture
def database() -> Database:
    return build_database()


@fixture
def seeded(src: Database) -> Seeded:  # `src` matches the test's `src`
    src.insert("ana", 1)
    return Seeded("ana")


def test_copy(src: Database, dst: Database, seeded: Seeded) -> None:
    saw(src, dst)
    copy(src, dst)
    assert dst.rows == src.rows
