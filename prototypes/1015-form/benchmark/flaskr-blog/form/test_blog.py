"""Flask's tutorial blog tests (flaskr `tests/test_blog.py`), ported to the form.

The tutorial's conftest chain `app` -> `client` -> `auth` is kept as fixtures. The database file
is a stock: each case samples a fresh file, and the runner fingerprints it around each check.
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

# read in SQL for populating test data
DATA_SQL = (Path(__file__).parent / "data.sql").read_bytes().decode("utf8")


# ── fixtures: the tutorial's conftest ──────────────────


class BlogDb(Samples[Path], Fingerprints[Path]):
    """A stock: a temporary database file of its own for each case."""

    def sample(self) -> Path:
        db_fd, db_path = tempfile.mkstemp()
        os.close(db_fd)
        return Path(db_path)

    def discard(self, value: Path, /) -> None:
        value.unlink()

    def fingerprint(self, value: Path, /) -> object:
        return value.read_bytes()


def app(db: Path = use(BlogDb)) -> Flask:
    """Create and configure a new app instance for each case."""
    app = create_app({"TESTING": True, "DATABASE": str(db)})
    # create the database and load test data
    with app.app_context():
        init_db()
        get_db().executescript(DATA_SQL)
    return app


def client(app: Flask = use(app)) -> FlaskClient:
    """A test client for the app."""
    return app.test_client()


class AuthActions:
    def __init__(self, client: FlaskClient) -> None:
        self._client = client

    def login(self, username: str = "test", password: str = "test") -> TestResponse:
        return self._client.post("/auth/login", data={"username": username, "password": password})

    def logout(self) -> TestResponse:
        return self._client.get("/auth/logout")


def auth(client: FlaskClient = use(client)) -> AuthActions:
    return AuthActions(client)


# ── the tests ─────────────────────────────


def test_index(client: FlaskClient = use(client), auth: AuthActions = use(auth)) -> None:
    anonymous = client.get("/")
    auth.login()
    logged_in = client.get("/")
    assert b"Log In" in anonymous.data
    assert b"Register" in anonymous.data
    assert b"test title" in logged_in.data
    assert b"by test on 2018-01-01" in logged_in.data
    assert b"test\nbody" in logged_in.data
    assert b'href="/1/update"' in logged_in.data


def test_login_required(path: str, client: FlaskClient = use(client)) -> None:
    response = client.post(path)
    assert response.headers["Location"] == "/auth/login"


login_required_create = case(test_login_required, path="/create")
login_required_update = case(test_login_required, path="/1/update")
login_required_delete = case(test_login_required, path="/1/delete")


def test_author_required(
    app: Flask = use(app), client: FlaskClient = use(client), auth: AuthActions = use(auth)
) -> None:
    # change the post author to another user
    with app.app_context():
        db = get_db()
        db.execute("UPDATE post SET author_id = 2 WHERE id = 1")
        db.commit()

    auth.login()
    update = client.post("/1/update")
    delete = client.post("/1/delete")
    index = client.get("/")
    # current user can't modify other user's post
    assert update.status_code == 403
    assert delete.status_code == 403
    # current user doesn't see edit link
    assert b'href="/1/update"' not in index.data


def test_exists_required(
    path: str, client: FlaskClient = use(client), auth: AuthActions = use(auth)
) -> None:
    auth.login()
    response = client.post(path)
    assert response.status_code == 404


exists_required_update = case(test_exists_required, path="/2/update")
exists_required_delete = case(test_exists_required, path="/2/delete")


def test_create(
    client: FlaskClient = use(client), auth: AuthActions = use(auth), app: Flask = use(app)
) -> None:
    auth.login()
    form = client.get("/create")
    client.post("/create", data={"title": "created", "body": ""})

    with app.app_context():
        db = get_db()
        count = db.execute("SELECT COUNT(id) FROM post").fetchone()[0]
        assert form.status_code == 200
        assert count == 2


def test_update(
    client: FlaskClient = use(client), auth: AuthActions = use(auth), app: Flask = use(app)
) -> None:
    auth.login()
    form = client.get("/1/update")
    client.post("/1/update", data={"title": "updated", "body": ""})

    with app.app_context():
        db = get_db()
        post = db.execute("SELECT * FROM post WHERE id = 1").fetchone()
        assert form.status_code == 200
        assert post["title"] == "updated"


def test_create_update_validate(
    path: str, client: FlaskClient = use(client), auth: AuthActions = use(auth)
) -> None:
    auth.login()
    response = client.post(path, data={"title": "", "body": ""})
    assert b"Title is required." in response.data


create_update_validate_create = case(test_create_update_validate, path="/create")
create_update_validate_update = case(test_create_update_validate, path="/1/update")


def test_delete(
    client: FlaskClient = use(client), auth: AuthActions = use(auth), app: Flask = use(app)
) -> None:
    auth.login()
    response = client.post("/1/delete")

    with app.app_context():
        db = get_db()
        post = db.execute("SELECT * FROM post WHERE id = 1").fetchone()
        assert response.headers["Location"] == "/"
        assert post is None
