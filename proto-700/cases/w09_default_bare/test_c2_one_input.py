from fixtures import database
from subject import Database, Tx


def test_insert(db: Tx[Database] = database) -> None:
    db.insert("a")
    assert db.rows() == ["a"], "the row is in the table"
