"""The refusal kept: the tester writes a type that holds both."""

from dataclasses import dataclass

from common import Seeded, build_database, copy, saw
from sketch import fixture
from subject import Database

POLICIES: dict[str, dict[str, bool]] = {"refuse": {}, "strict": {"strict": True}}


@dataclass(frozen=True)
class CopyBench:
    src: Database
    dst: Database


@fixture
def database() -> Database:
    return build_database()


@fixture
def bench() -> CopyBench:
    return CopyBench(Database(), Database())  # bypasses the database builder


@fixture
def seeded(bench: CopyBench) -> Seeded:
    bench.src.insert("ana", 1)
    return Seeded("ana")


def test_copy(bench: CopyBench, seeded: Seeded) -> None:
    saw(bench.src, bench.dst)
    copy(bench.src, bench.dst)
    assert bench.dst.rows == bench.src.rows
