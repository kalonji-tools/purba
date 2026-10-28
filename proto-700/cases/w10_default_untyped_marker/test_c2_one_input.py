from fixtures import database
from sketch import use_any
from subject import Database, Tx


def test_insert(db: Tx[Database] = use_any(database)) -> None:
    db.insert("a")
    assert db.rows() == ["a"], "the row is in the table"
