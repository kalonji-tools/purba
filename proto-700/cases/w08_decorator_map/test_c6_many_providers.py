from fixtures import postgres, sqlite
from sketch import each, using
from subject import Database, Tx


@using(engine=each(sqlite, postgres))
def test_roundtrip(engine: Tx[Database]) -> None:
    engine.insert("a")
    assert engine.rows() == ["a"], "the row is in the table"
