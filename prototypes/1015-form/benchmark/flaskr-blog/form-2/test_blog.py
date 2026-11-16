"""The flaskr tutorial's test_blog.py, in the form. Each test takes the tutorial's `app`, `client`
and `auth` from flaskr_fixtures.py; in a case, all three share one app and one database."""

from __future__ import annotations

from flask import Flask
from flask.testing import FlaskClient

from cases import case, use
from flaskr.db import get_db
from flaskr_fixtures import App, AuthActions, auth, client


def test_index(client: FlaskClient = use(client), auth: AuthActions = use(auth)) -> None:
    anonymous = client.get("/")
    auth.login()
    response = client.get("/")
    assert b"Log In" in anonymous.data
    assert b"Register" in anonymous.data
    assert b"test title" in response.data
    assert b"by test on 2018-01-01" in response.data
    assert b"test\nbody" in response.data
    assert b'href="/1/update"' in response.data


def test_login_required(path: str, client: FlaskClient = use(client)) -> None:
    response = client.post(path)
    assert response.headers["Location"] == "/auth/login"


login_required_create = case(test_login_required, path="/create")
login_required_update = case(test_login_required, path="/1/update")
login_required_delete = case(test_login_required, path="/1/delete")


def test_author_required(
    app: Flask = use(App), client: FlaskClient = use(client), auth: AuthActions = use(auth)
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
    client: FlaskClient = use(client), auth: AuthActions = use(auth), app: Flask = use(App)
) -> None:
    auth.login()
    page = client.get("/create")
    client.post("/create", data={"title": "created", "body": ""})

    with app.app_context():
        db = get_db()
        count = db.execute("SELECT COUNT(id) FROM post").fetchone()[0]
    assert page.status_code == 200
    assert count == 2


def test_update(
    client: FlaskClient = use(client), auth: AuthActions = use(auth), app: Flask = use(App)
) -> None:
    auth.login()
    page = client.get("/1/update")
    client.post("/1/update", data={"title": "updated", "body": ""})

    with app.app_context():
        db = get_db()
        post = db.execute("SELECT * FROM post WHERE id = 1").fetchone()
    assert page.status_code == 200
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
    client: FlaskClient = use(client), auth: AuthActions = use(auth), app: Flask = use(App)
) -> None:
    auth.login()
    response = client.post("/1/delete")

    with app.app_context():
        db = get_db()
        post = db.execute("SELECT * FROM post WHERE id = 1").fetchone()
    assert response.headers["Location"] == "/"
    assert post is None
