from fixtures import client, database
from sketch import using
from subject import Client, Database, Tx


@using(client=client, db=database)
def test_fetch(client: Client, db: Tx[Database]) -> None:
    db.insert("a")
    assert client.fetch(db) == ["a"], "the client reads the row"
