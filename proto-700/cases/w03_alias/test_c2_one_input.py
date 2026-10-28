from fixtures import DbTx


def test_insert(db: DbTx) -> None:
    db.insert("a")
    assert db.rows() == ["a"], "the row is in the table"
