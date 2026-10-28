from fixtures import DbTx
from subject import Client, Clock, Mailer, Settings


def test_signup(client: Client, db: DbTx, settings: Settings, clock: Clock, mailer: Mailer) -> None:
    client.signup(db, mailer, clock, "ada")
    assert settings.name == "test", "the client runs with test settings"
    assert db.rows() == ["ada@0"], "the user is stored"
    assert mailer.outbox == ["welcome ada"], "the user is welcomed"
