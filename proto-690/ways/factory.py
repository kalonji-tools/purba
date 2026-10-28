"""The test names a factory and calls it once for each database."""

from collections.abc import Callable

from common import build_database, copy, saw
from sketch import fixture
from subject import Database

POLICIES: dict[str, dict[str, bool]] = {"refuse": {}, "strict": {"strict": True}}


@fixture
def database() -> Database:
    return build_database()


# No builder can seed the source: it is made inside the test.
def test_copy(new_db: Callable[[], Database]) -> None:
    src, dst = new_db(), new_db()
    src.insert("ana", 1)
    saw(src, dst)
    copy(src, dst)
    assert dst.rows == src.rows
