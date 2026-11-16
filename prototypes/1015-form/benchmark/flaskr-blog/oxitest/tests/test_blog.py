"""Tests for flaskr's blog blueprint.

Ported from flaskr's tests/test_blog.py (BSD-3-Clause, see LICENSE.flask).
"""

from dataclasses import dataclass

import oxitest as oxi
from flask import Flask
from flask.testing import FlaskClient
from flaskr.db import get_db
from oxitest import Fixture

from tests.auth_actions import AuthActions


@dataclass(frozen=True)
class Route:
    path: str


def test_index(client: Fixture[FlaskClient], auth: Fixture[AuthActions]) -> None:
    response = client.get("/")
    assert b"Log In" in response.data, "an anonymous visitor is offered a login link"
    assert b"Register" in response.data, "an anonymous visitor is offered a register link"

    auth.login()
    response = client.get("/")
    assert b"test title" in response.data, "the index lists the post's title"
    assert b"by test on 2018-01-01" in response.data, "the index names author and date"
    assert b"test\nbody" in response.data, "the index shows the post's body"
    assert b'href="/1/update"' in response.data, "the author sees an edit link"


@oxi.parametrize(
    create=Route(path="/create"),
    update=Route(path="/1/update"),
    delete=Route(path="/1/delete"),
)
def test_login_required(client: Fixture[FlaskClient], path: str) -> None:
    response = client.post(path)
    assert response.headers["Location"] == "/auth/login", (
        "an anonymous post is redirected to the login page"
    )


def test_author_required(
    app: Fixture[Flask], client: Fixture[FlaskClient], auth: Fixture[AuthActions]
) -> None:
    # change the post author to another user
    with app.app_context():
        db = get_db()
        db.execute("UPDATE post SET author_id = 2 WHERE id = 1")
        db.commit()

    auth.login()
    # current user can't modify other user's post
    assert client.post("/1/update").status_code == 403, (
        "only the author may update a post"
    )
    assert client.post("/1/delete").status_code == 403, (
        "only the author may delete a post"
    )
    # current user doesn't see edit link
    assert b'href="/1/update"' not in client.get("/").data, (
        "a user who is not the author sees no edit link"
    )


@oxi.parametrize(
    update=Route(path="/2/update"),
    delete=Route(path="/2/delete"),
)
def test_exists_required(
    client: Fixture[FlaskClient], auth: Fixture[AuthActions], path: str
) -> None:
    auth.login()
    assert client.post(path).status_code == 404, "a post that does not exist is a 404"


def test_create(
    client: Fixture[FlaskClient], auth: Fixture[AuthActions], app: Fixture[Flask]
) -> None:
    auth.login()
    assert client.get("/create").status_code == 200, "the create form renders"
    client.post("/create", data={"title": "created", "body": ""})

    with app.app_context():
        db = get_db()
        count = db.execute("SELECT COUNT(id) FROM post").fetchone()[0]
        assert count == 2, "the new post is stored beside the seeded one"


def test_update(
    client: Fixture[FlaskClient], auth: Fixture[AuthActions], app: Fixture[Flask]
) -> None:
    auth.login()
    assert client.get("/1/update").status_code == 200, "the update form renders"
    client.post("/1/update", data={"title": "updated", "body": ""})

    with app.app_context():
        db = get_db()
        post = db.execute("SELECT * FROM post WHERE id = 1").fetchone()
        assert post["title"] == "updated", "the stored title is the new one"


@oxi.parametrize(
    create=Route(path="/create"),
    update=Route(path="/1/update"),
)
def test_create_update_validate(
    client: Fixture[FlaskClient], auth: Fixture[AuthActions], path: str
) -> None:
    auth.login()
    response = client.post(path, data={"title": "", "body": ""})
    assert b"Title is required." in response.data, "an empty title is refused"


def test_delete(
    client: Fixture[FlaskClient], auth: Fixture[AuthActions], app: Fixture[Flask]
) -> None:
    auth.login()
    response = client.post("/1/delete")
    assert response.headers["Location"] == "/", "a delete redirects to the index"

    with app.app_context():
        db = get_db()
        post = db.execute("SELECT * FROM post WHERE id = 1").fetchone()
        assert post is None, "the deleted post is gone from the database"
