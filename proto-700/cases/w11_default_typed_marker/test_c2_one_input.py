from fixtures import database, tx
from sketch import use
from subject import Database, Tx


def test_insert(db: Tx[Database] = use(tx, database)) -> None:
    db.insert("a")
    assert db.rows() == ["a"], "the row is in the table"
