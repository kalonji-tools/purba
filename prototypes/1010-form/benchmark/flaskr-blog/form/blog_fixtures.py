"""The fixtures of the blog tests: an app on a database file of its own, its client, and its auth."""

from __future__ import annotations

import os
import tempfile
from dataclasses import dataclass
from pathlib import Path

from flask import Flask
from flask.testing import FlaskClient
from werkzeug.test import TestResponse

from cases import Fingerprints, Samples
from flaskr import create_app
from flaskr.db import get_db, init_db

# read in SQL for populating test data
_data_sql = (Path(__file__).parent / "data.sql").read_bytes().decode("utf8")


class AuthActions:
    def __init__(self, client: FlaskClient) -> None:
        self._client = client

    def login(self, username: str = "test", password: str = "test") -> TestResponse:
        return self._client.post("/auth/login", data={"username": username, "password": password})

    def logout(self) -> TestResponse:
        return self._client.get("/auth/logout")


@dataclass
class Blog:
    """The app, a test client for it, and the auth actions of that client."""

    app: Flask
    client: FlaskClient
    auth: AuthActions


class BlogDb(Samples[Blog], Fingerprints[Blog]):
    """A stock: a new app on a temporary database file, loaded with the test data, for each case."""

    def sample(self) -> Blog:
        # create a temporary file to isolate the database for each case
        self.db_fd, db_path = tempfile.mkstemp()
        # create the app with common test config
        app = create_app({"TESTING": True, "DATABASE": db_path})

        # create the database and load test data
        with app.app_context():
            init_db()
            get_db().executescript(_data_sql)

        client = app.test_client()
        return Blog(app=app, client=client, auth=AuthActions(client))

    def discard(self, value: Blog, /) -> None:
        # close and remove the temporary database
        os.close(self.db_fd)
        os.unlink(value.app.config["DATABASE"])

    def fingerprint(self, value: Blog, /) -> object:
        return Path(value.app.config["DATABASE"]).read_bytes()
