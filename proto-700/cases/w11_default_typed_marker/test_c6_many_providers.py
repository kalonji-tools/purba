from fixtures import postgres, sqlite, tx
from sketch import each, use
from subject import Database, Tx


def test_roundtrip(engine: Tx[Database] = use(tx, each(sqlite, postgres))) -> None:
    engine.insert("a")
    assert engine.rows() == ["a"], "the row is in the table"
