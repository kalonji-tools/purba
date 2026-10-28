from typing import Annotated

from fixtures import client, clock, database, mailer, settings
from sketch import From
from subject import Client, Clock, Database, Mailer, Settings, Tx


def test_signup(
    client: Annotated[Client, From(client)],
    db: Annotated[Tx[Database], From(database)],
    settings: Annotated[Settings, From(settings)],
    clock: Annotated[Clock, From(clock)],
    mailer: Annotated[Mailer, From(mailer)],
) -> None:
    client.signup(db, mailer, clock, "ada")
    assert settings.name == "test", "the client runs with test settings"
    assert db.rows() == ["ada@0"], "the user is stored"
    assert mailer.outbox == ["welcome ada"], "the user is welcomed"
