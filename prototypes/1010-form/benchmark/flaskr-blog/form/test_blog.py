"""The blog: the index, who may create, update and delete a post, and what each change leaves."""

from __future__ import annotations

from blog_fixtures import Blog, BlogDb
from cases import case, use
from flaskr.db import get_db


def test_index(blog: Blog = use(BlogDb)) -> None:
    anonymous = blog.client.get("/")
    blog.auth.login()
    logged_in = blog.client.get("/")
    assert b"Log In" in anonymous.data
    assert b"Register" in anonymous.data
    assert b"test title" in logged_in.data
    assert b"by test on 2018-01-01" in logged_in.data
    assert b"test\nbody" in logged_in.data
    assert b'href="/1/update"' in logged_in.data


def test_login_required(path: str, blog: Blog = use(BlogDb)) -> None:
    response = blog.client.post(path)
    assert response.headers["Location"] == "/auth/login"


login_required_create = case(test_login_required, path="/create")
login_required_update = case(test_login_required, path="/1/update")
login_required_delete = case(test_login_required, path="/1/delete")


def test_author_required(blog: Blog = use(BlogDb)) -> None:
    # change the post author to another user
    with blog.app.app_context():
        db = get_db()
        db.execute("UPDATE post SET author_id = 2 WHERE id = 1")
        db.commit()

    blog.auth.login()
    update = blog.client.post("/1/update")
    delete = blog.client.post("/1/delete")
    index = blog.client.get("/")
    # current user can't modify other user's post
    assert update.status_code == 403
    assert delete.status_code == 403
    # current user doesn't see edit link
    assert b'href="/1/update"' not in index.data


def test_exists_required(path: str, blog: Blog = use(BlogDb)) -> None:
    blog.auth.login()
    response = blog.client.post(path)
    assert response.status_code == 404


exists_required_update = case(test_exists_required, path="/2/update")
exists_required_delete = case(test_exists_required, path="/2/delete")


def test_create(blog: Blog = use(BlogDb)) -> None:
    blog.auth.login()
    form = blog.client.get("/create")
    blog.client.post("/create", data={"title": "created", "body": ""})

    with blog.app.app_context():
        count = get_db().execute("SELECT COUNT(id) FROM post").fetchone()[0]
    assert form.status_code == 200
    assert count == 2


def test_update(blog: Blog = use(BlogDb)) -> None:
    blog.auth.login()
    form = blog.client.get("/1/update")
    blog.client.post("/1/update", data={"title": "updated", "body": ""})

    with blog.app.app_context():
        post = get_db().execute("SELECT * FROM post WHERE id = 1").fetchone()
    assert form.status_code == 200
    assert post["title"] == "updated"


def test_create_update_validate(path: str, blog: Blog = use(BlogDb)) -> None:
    blog.auth.login()
    response = blog.client.post(path, data={"title": "", "body": ""})
    assert b"Title is required." in response.data


create_update_validate_create = case(test_create_update_validate, path="/create")
create_update_validate_update = case(test_create_update_validate, path="/1/update")


def test_delete(blog: Blog = use(BlogDb)) -> None:
    blog.auth.login()
    response = blog.client.post("/1/delete")

    with blog.app.app_context():
        post = get_db().execute("SELECT * FROM post WHERE id = 1").fetchone()
    assert response.headers["Location"] == "/"
    assert post is None
