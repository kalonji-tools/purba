"""The blog tests of Flask's tutorial (flaskr)."""

from __future__ import annotations

import tempfile
from pathlib import Path

from cases import case, use
from flask import Flask
from flask.testing import FlaskClient
from flaskr import create_app
from flaskr.db import get_db, init_db

DATA = (Path(__file__).parent / "data.sql").read_text()


def app_with_data() -> Flask:
    """A new app on a temporary database, with the schema and the test rows."""
    _, path = tempfile.mkstemp()
    app = create_app({"TESTING": True, "DATABASE": path})
    with app.app_context():
        init_db()
        get_db().executescript(DATA)
    return app


def client_with_data() -> FlaskClient:
    """A test client of `app_with_data()`."""
    return app_with_data().test_client()


def login(client: FlaskClient) -> None:
    client.post("/auth/login", data={"username": "test", "password": "test"})


def test_index_logged_out(client: FlaskClient = use(client_with_data)) -> None:
    response = client.get("/")
    assert b"Log In" in response.data
    assert b"Register" in response.data


def test_index_logged_in(client: FlaskClient = use(client_with_data)) -> None:
    login(client)
    response = client.get("/")
    assert b"test title" in response.data
    assert b"by test on 2018-01-01" in response.data
    assert b"test\nbody" in response.data
    assert b'href="/1/update"' in response.data


def test_login_required(path: str, client: FlaskClient = use(client_with_data)) -> None:
    response = client.post(path)
    assert response.headers["Location"] == "/auth/login"


create_needs_login = case(test_login_required, path="/create")
update_needs_login = case(test_login_required, path="/1/update")
delete_needs_login = case(test_login_required, path="/1/delete")


def test_author_required(client: FlaskClient = use(client_with_data)) -> None:
    with client.application.app_context():
        db = get_db()
        db.execute("UPDATE post SET author_id = 2 WHERE id = 1")
        db.commit()
    login(client)
    update = client.post("/1/update")
    delete = client.post("/1/delete")
    index = client.get("/")
    assert update.status_code == 403
    assert delete.status_code == 403
    assert b'href="/1/update"' not in index.data


def test_exists_required(path: str, client: FlaskClient = use(client_with_data)) -> None:
    login(client)
    response = client.post(path)
    assert response.status_code == 404


update_missing = case(test_exists_required, path="/2/update")
delete_missing = case(test_exists_required, path="/2/delete")


def test_create(client: FlaskClient = use(client_with_data)) -> None:
    login(client)
    page = client.get("/create")
    client.post("/create", data={"title": "created", "body": ""})
    with client.application.app_context():
        count = get_db().execute("SELECT COUNT(id) FROM post").fetchone()[0]
    assert page.status_code == 200
    assert count == 2


def test_update(client: FlaskClient = use(client_with_data)) -> None:
    login(client)
    page = client.get("/1/update")
    client.post("/1/update", data={"title": "updated", "body": ""})
    with client.application.app_context():
        post = get_db().execute("SELECT * FROM post WHERE id = 1").fetchone()
    assert page.status_code == 200
    assert post["title"] == "updated"


def test_create_update_validate(path: str, client: FlaskClient = use(client_with_data)) -> None:
    login(client)
    response = client.post(path, data={"title": "", "body": ""})
    assert b"Title is required." in response.data


create_validates = case(test_create_update_validate, path="/create")
update_validates = case(test_create_update_validate, path="/1/update")


def test_delete(client: FlaskClient = use(client_with_data)) -> None:
    login(client)
    response = client.post("/1/delete")
    with client.application.app_context():
        post = get_db().execute("SELECT * FROM post WHERE id = 1").fetchone()
    assert response.headers["Location"] == "/"
    assert post is None
