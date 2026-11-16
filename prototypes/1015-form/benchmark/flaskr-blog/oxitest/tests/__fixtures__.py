"""Fixtures for the flaskr blog tests.

Ported from flaskr's tests/conftest.py (BSD-3-Clause, see LICENSE.flask).
The original ``runner`` fixture is not ported: no blog test uses it.
"""

from pathlib import Path

import oxitest as oxi
from flask import Flask
from flask.testing import FlaskClient
from flaskr import create_app
from flaskr.db import get_db
from flaskr.db import init_db
from oxitest import Fixture
from oxitest import TempDir

from tests.auth_actions import AuthActions

# read in SQL for populating test data
_data_sql = (Path(__file__).parent / "data.sql").read_text(encoding="utf8")


@oxi.fixture(lifetime="function")
def app(tmp: TempDir) -> Flask:
    """Create and configure a new app instance for each test."""
    # The database file lives in a per-fixture temporary directory, which
    # oxitest removes after the test.
    app = create_app({"TESTING": True, "DATABASE": str(tmp.path / "flaskr.sqlite")})

    # create the database and load test data
    with app.app_context():
        init_db()
        get_db().executescript(_data_sql)

    return app


@oxi.fixture(lifetime="function")
def client(app: Fixture[Flask]) -> FlaskClient:
    """A test client for the app."""
    return app.test_client()


@oxi.fixture(lifetime="function")
def auth(client: Fixture[FlaskClient]) -> AuthActions:
    """Log in and out through the same client the test uses."""
    return AuthActions(client)
