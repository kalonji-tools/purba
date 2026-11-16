"""Log in and out through the app's own routes.

Ported from flaskr's tests/conftest.py (BSD-3-Clause, see LICENSE.flask).
This is a plain class, not a fixture: the ``auth`` fixture in
``__fixtures__.py`` builds one around the test's client, and the test module
imports the class only to annotate that fixture.
"""

from flask.testing import FlaskClient
from werkzeug.test import TestResponse


class AuthActions:
    def __init__(self, client: FlaskClient) -> None:
        self._client = client

    def login(self, username: str = "test", password: str = "test") -> TestResponse:
        return self._client.post(
            "/auth/login", data={"username": username, "password": password}
        )

    def logout(self) -> TestResponse:
        return self._client.get("/auth/logout")
