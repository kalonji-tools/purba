from fixtures import client, clock, database, mailer, settings
from sketch import use_any
from subject import Client, Clock, Database, Mailer, Settings, Tx


def test_signup(
    client: Client = use_any(client),
    db: Tx[Database] = use_any(database),
    settings: Settings = use_any(settings),
    clock: Clock = use_any(clock),
    mailer: Mailer = use_any(mailer),
) -> None:
    client.signup(db, mailer, clock, "ada")
    assert settings.name == "test", "the client runs with test settings"
    assert db.rows() == ["ada@0"], "the user is stored"
    assert mailer.outbox == ["welcome ada"], "the user is welcomed"
