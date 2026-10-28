from fixtures import EngineDb
from subject import Tx


def test_roundtrip(engine: Tx[EngineDb]) -> None:
    engine.insert("a")
    assert engine.rows() == ["a"], "the row is in the table"
