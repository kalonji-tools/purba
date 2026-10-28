from fixtures import client, clock, database, mailer, settings, tx
from sketch import use
from subject import Client, Clock, Database, Mailer, Settings, Tx


def test_signup(
    client: Client = use(client),
    db: Tx[Database] = use(tx, database),
    settings: Settings = use(settings),
    clock: Clock = use(clock),
    mailer: Mailer = use(mailer),
) -> None:
    client.signup(db, mailer, clock, "ada")
    assert settings.name == "test", "the client runs with test settings"
    assert db.rows() == ["ada@0"], "the user is stored"
    assert mailer.outbox == ["welcome ada"], "the user is welcomed"
