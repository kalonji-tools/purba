from fixtures import postgres, sqlite
from sketch import each
from subject import Database, Tx


def test_roundtrip(engine: Tx[Database] = each(sqlite, postgres)) -> None:
    engine.insert("a")
    assert engine.rows() == ["a"], "the row is in the table"
