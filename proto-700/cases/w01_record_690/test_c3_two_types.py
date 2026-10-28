from subject import Client, Database, Tx


def test_fetch(client: Client, db: Tx[Database]) -> None:
    db.insert("a")
    assert client.fetch(db) == ["a"], "the client reads the row"
