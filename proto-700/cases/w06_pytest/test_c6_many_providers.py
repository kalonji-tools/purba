from subject import Database, Tx


def test_roundtrip(engine: Tx[Database]) -> None:
    engine.insert("a")
    assert engine.rows() == ["a"], "the row is in the table"
