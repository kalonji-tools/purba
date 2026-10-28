from fixtures import client, database
from sketch import use_any
from subject import Client, Database, Tx


def test_fetch(client: Client = use_any(client), db: Tx[Database] = use_any(database)) -> None:
    db.insert("a")
    assert client.fetch(db) == ["a"], "the client reads the row"
