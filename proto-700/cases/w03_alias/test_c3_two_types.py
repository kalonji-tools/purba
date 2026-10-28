from fixtures import DbTx
from subject import Client


def test_fetch(client: Client, db: DbTx) -> None:
    db.insert("a")
    assert client.fetch(db) == ["a"], "the client reads the row"
