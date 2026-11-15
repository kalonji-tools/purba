"""The blog of Flask's tutorial app, flaskr.

A chain of two stages carries the arrange that the tests share: `seeded` makes the app on a
database of the schema and the rows of data.sql, and `logged_in` logs the user `test` in on a
client of that app. Each stage checks its own step once; a test that takes it does not repeat it.
"""

from __future__ import annotations

import os
import tempfile
from pathlib import Path

from flask import Flask
from flask.testing import FlaskClient
from werkzeug.test import TestResponse

from cases import Fingerprints, Samples, case, use
from flaskr import create_app
from flaskr.db import get_db, init_db

DATA_SQL = Path(__file__).with_name("data.sql").read_text(encoding="utf8")


# ── fixtures ────────────────────────────


class TempDb(Samples[Path], Fingerprints[Path]):
    """A stock: an empty database file of its own for each case, removed after it."""

    def sample(self) -> Path:
        handle, path = tempfile.mkstemp()
        os.close(handle)
        return Path(path)

    def discard(self, value: Path, /) -> None:
        value.unlink()

    def fingerprint(self, value: Path, /) -> object:
        return value.read_bytes()


class AuthActions:
    """Logs a user in on a client, as the original `auth` fixture does."""

    def __init__(self, client: FlaskClient) -> None:
        self._client = client

    def login(self, username: str = "test", password: str = "test") -> TestResponse:
        return self._client.post("/auth/login", data={"username": username, "password": password})


def user_id_of(client: FlaskClient) -> object:
    """The user id that the client's session holds."""
    with client.session_transaction() as session:
        return session.get("user_id")


# ── stages ──────────────────────────────


def test_seed_app(db: Path = use(TempDb)) -> Flask:
    """A stage: the app of each test, on a database of the schema and the rows of data.sql."""
    app = create_app({"TESTING": True, "DATABASE": str(db)})
    with app.app_context():
        init_db()
        get_db().executescript(DATA_SQL)
        users = get_db().execute("SELECT id, username FROM user ORDER BY id").fetchall()
        posts = get_db().execute("SELECT id, author_id, title, body FROM post").fetchall()
        assert [tuple(user) for user in users] == [(1, "test"), (2, "other")]
        assert [tuple(post) for post in posts] == [(1, 1, "test title", "test\nbody")]
    return app


seeded = case(test_seed_app)


def test_login(app: Flask = use(seeded)) -> FlaskClient:
    """A stage: the user `test` logs in, and the client keeps the session."""
    client = app.test_client()
    response = AuthActions(client).login()
    assert response.headers.get("Location") == "/"
    assert user_id_of(client) == 1
    return client


logged_in = case(test_login)


# ── the blog ────────────────────────────


def test_index(client: FlaskClient = use(logged_in)) -> None:
    anonymous = client.application.test_client().get("/")
    response = client.get("/")
    assert b"Log In" in anonymous.data
    assert b"Register" in anonymous.data
    assert b"test title" in response.data
    assert b"by test on 2018-01-01" in response.data
    assert b"test\nbody" in response.data
    assert b'href="/1/update"' in response.data


def test_login_required(path: str, app: Flask = use(seeded)) -> None:
    response = app.test_client().post(path)
    assert response.headers["Location"] == "/auth/login"


anonymous_create = case(test_login_required, path="/create")
anonymous_update = case(test_login_required, path="/1/update")
anonymous_delete = case(test_login_required, path="/1/delete")


def test_author_required(client: FlaskClient = use(logged_in)) -> None:
    # change the post author to another user
    with client.application.app_context():
        db = get_db()
        db.execute("UPDATE post SET author_id = 2 WHERE id = 1")
        db.commit()
    update = client.post("/1/update")
    delete = client.post("/1/delete")
    index = client.get("/")
    # current user can't modify other user's post
    assert update.status_code == 403
    assert delete.status_code == 403
    # current user doesn't see edit link
    assert b'href="/1/update"' not in index.data


def test_exists_required(path: str, client: FlaskClient = use(logged_in)) -> None:
    response = client.post(path)
    assert response.status_code == 404


missing_update = case(test_exists_required, path="/2/update")
missing_delete = case(test_exists_required, path="/2/delete")


def test_create(client: FlaskClient = use(logged_in)) -> None:
    page = client.get("/create")
    client.post("/create", data={"title": "created", "body": ""})
    with client.application.app_context():
        count = get_db().execute("SELECT COUNT(id) FROM post").fetchone()[0]
        assert page.status_code == 200
        assert count == 2


def test_update(client: FlaskClient = use(logged_in)) -> None:
    page = client.get("/1/update")
    client.post("/1/update", data={"title": "updated", "body": ""})
    with client.application.app_context():
        post = get_db().execute("SELECT * FROM post WHERE id = 1").fetchone()
        assert page.status_code == 200
        assert post["title"] == "updated"


def test_create_update_validate(path: str, client: FlaskClient = use(logged_in)) -> None:
    response = client.post(path, data={"title": "", "body": ""})
    assert b"Title is required." in response.data


validate_create = case(test_create_update_validate, path="/create")
validate_update = case(test_create_update_validate, path="/1/update")


def test_delete(client: FlaskClient = use(logged_in)) -> None:
    response = client.post("/1/delete")
    with client.application.app_context():
        post = get_db().execute("SELECT * FROM post WHERE id = 1").fetchone()
        assert response.headers["Location"] == "/"
        assert post is None
