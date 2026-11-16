"""The tutorial's conftest, in the form: `app` is a stock (its database is a file outside the
process), `client` uses `app`, and `auth` uses `client`."""

from __future__ import annotations

import hashlib
import os
import tempfile
from pathlib import Path

from flask import Flask
from flask.testing import FlaskClient
from werkzeug.test import TestResponse

from cases import Fingerprints, Samples, use
from flaskr import create_app
from flaskr.db import get_db, init_db

# read in SQL for populating test data
_data_sql = (Path(__file__).parent / "data.sql").read_bytes().decode("utf8")


class App(Samples[Flask], Fingerprints[Flask]):
    """Create and configure a new app instance for each case, on a database file of its own."""

    def sample(self) -> Flask:
        # create a temporary file to isolate the database for each case
        db_fd, db_path = tempfile.mkstemp()
        os.close(db_fd)  # sqlite opens the file by its path
        # create the app with common test config
        app = create_app({"TESTING": True, "DATABASE": db_path})

        # create the database and load test data
        with app.app_context():
            init_db()
            get_db().executescript(_data_sql)

        return app

    def discard(self, value: Flask, /) -> None:
        # remove the temporary database
        os.unlink(value.config["DATABASE"])

    def fingerprint(self, value: Flask, /) -> object:
        return hashlib.sha256(Path(value.config["DATABASE"]).read_bytes()).hexdigest()


def client(app: Flask = use(App)) -> FlaskClient:
    """A test client for the app."""
    return app.test_client()


class AuthActions:
    def __init__(self, client: FlaskClient) -> None:
        self._client = client

    def login(self, username: str = "test", password: str = "test") -> TestResponse:
        return self._client.post(
            "/auth/login", data={"username": username, "password": password}
        )

    def logout(self) -> TestResponse:
        return self._client.get("/auth/logout")


def auth(client: FlaskClient = use(client)) -> AuthActions:
    return AuthActions(client)
