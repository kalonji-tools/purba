from fixtures import client, database
from subject import Client, Database, Tx


def test_fetch(client: Client = client, db: Tx[Database] = database) -> None:
    db.insert("a")
    assert client.fetch(db) == ["a"], "the client reads the row"
