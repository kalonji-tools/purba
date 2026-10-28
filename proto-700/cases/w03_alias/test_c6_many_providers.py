from fixtures import EngineTx


def test_roundtrip(engine: EngineTx) -> None:
    engine.insert("a")
    assert engine.rows() == ["a"], "the row is in the table"
