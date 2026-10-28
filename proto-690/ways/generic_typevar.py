"""As generic_derived, with an old-style TypeVar in place of `def tx[D: Database]`."""

from typing import NewType, TypeVar

from adapter import Tx
from common import Seeded, build_database, copy, saw
from sketch import fixture, resource
from subject import Database

POLICIES: dict[str, dict[str, bool]] = {"refuse": {}, "strict": {"strict": True}}

SourceDb = NewType("SourceDb", Database)
TargetDb = NewType("TargetDb", Database)
D = TypeVar("D", bound=Database)


@resource
def database() -> Database:
    return build_database()


@fixture
def tx(db: D) -> Tx[D]:
    return Tx(db)


@fixture
def seeded(src: Tx[SourceDb]) -> Seeded:
    src.db.insert("ana", 1)
    return Seeded("ana")


def test_copy(src: Tx[SourceDb], dst: Tx[TargetDb], seeded: Seeded) -> None:
    saw(src.db, dst.db)
    copy(src.db, dst.db)
    assert dst.db.rows == src.db.rows


def test_starts_clean(src: Tx[SourceDb], dst: Tx[TargetDb]) -> None:
    saw(src.db, dst.db)
    assert src.db.rows == {} and dst.db.rows == {}
