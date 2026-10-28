from typing import Annotated

from fixtures import postgres, sqlite
from sketch import From, each
from subject import Database, Tx


def test_roundtrip(engine: Annotated[Tx[Database], From(each(sqlite, postgres))]) -> None:
    engine.insert("a")
    assert engine.rows() == ["a"], "the row is in the table"
