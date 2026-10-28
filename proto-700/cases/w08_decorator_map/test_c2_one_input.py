from fixtures import database
from sketch import using
from subject import Database, Tx


@using(db=database)
def test_insert(db: Tx[Database]) -> None:
    db.insert("a")
    assert db.rows() == ["a"], "the row is in the table"
