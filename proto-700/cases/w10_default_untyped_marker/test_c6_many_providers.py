from fixtures import postgres, sqlite
from sketch import each, use_any
from subject import Database, Tx


def test_roundtrip(engine: Tx[Database] = use_any(each(sqlite, postgres))) -> None:
    engine.insert("a")
    assert engine.rows() == ["a"], "the row is in the table"
