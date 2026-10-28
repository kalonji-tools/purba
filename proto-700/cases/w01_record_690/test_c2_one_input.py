from subject import Database, Tx


def test_insert(db: Tx[Database]) -> None:
    db.insert("a")
    assert db.rows() == ["a"], "the row is in the table"
