from typing import Annotated

from fixtures import client, database
from sketch import From
from subject import Client, Database, Tx


def test_fetch(
    client: Annotated[Client, From(client)],
    db: Annotated[Tx[Database], From(database)],
) -> None:
    db.insert("a")
    assert client.fetch(db) == ["a"], "the client reads the row"
