from fixtures import client, database, tx
from sketch import use
from subject import Client, Database, Tx


def test_fetch(client: Client = use(client), db: Tx[Database] = use(tx, database)) -> None:
    db.insert("a")
    assert client.fetch(db) == ["a"], "the client reads the row"
