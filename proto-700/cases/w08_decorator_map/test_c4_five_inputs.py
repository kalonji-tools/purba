from fixtures import client, clock, database, mailer, settings
from sketch import using
from subject import Client, Clock, Database, Mailer, Settings, Tx


@using(client=client, db=database, settings=settings, clock=clock, mailer=mailer)
def test_signup(
    client: Client,
    db: Tx[Database],
    settings: Settings,
    clock: Clock,
    mailer: Mailer,
) -> None:
    client.signup(db, mailer, clock, "ada")
    assert settings.name == "test", "the client runs with test settings"
    assert db.rows() == ["ada@0"], "the user is stored"
    assert mailer.outbox == ["welcome ada"], "the user is welcomed"
