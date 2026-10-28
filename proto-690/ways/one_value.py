"""The problem #689 refuses: two inputs of one type, one value per test."""

from common import Seeded, build_database, copy, saw
from sketch import fixture
from subject import Database

POLICIES = {"refused (#689)": {}, "allowed, one value": {"refuse": False}}


@fixture
def database() -> Database:
    return build_database()


@fixture
def seeded(src: Database) -> Seeded:
    src.insert("ana", 1)
    return Seeded("ana")


def test_copy(src: Database, dst: Database, seeded: Seeded) -> None:
    saw(src, dst)
    copy(src, dst)
    assert dst.rows == src.rows
