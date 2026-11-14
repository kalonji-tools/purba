"""8: a stock, an input whose state lives outside the process. Its fixture offers a sample, or a checkpoint."""

import shutil
import sqlite3
import tempfile
from pathlib import Path

from app import Outbox, User, sign_up, user_names
from cases import Checkpoints, Samples, case, raises, use

SEED = Path(tempfile.gettempdir()) / "prototype-3-users.db"


def seed() -> None:
    db = sqlite3.connect(SEED)
    db.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT UNIQUE)")
    db.execute("DELETE FROM users")
    db.execute("INSERT INTO users (name) VALUES ('old')")
    db.commit()
    db.close()


seed()


class UsersDb(Samples[sqlite3.Connection]):
    """A copy of the seeded file for each case, deleted after it."""

    def sample(self) -> sqlite3.Connection:
        copy = Path(tempfile.mkdtemp()) / "users.db"
        shutil.copy(SEED, copy)
        return sqlite3.connect(copy, isolation_level=None)

    def discard(self, value: sqlite3.Connection, /) -> None:
        (path,) = [row[2] for row in value.execute("PRAGMA database_list")]
        value.close()
        Path(path).unlink()


class SharedDb(Checkpoints[sqlite3.Connection, int]):
    """The seeded file itself: each case and each check restores it to a checkpoint."""

    def open(self) -> sqlite3.Connection:
        return sqlite3.connect(SEED, isolation_level=None)

    def checkpoint(self, value: sqlite3.Connection, /) -> int:
        return int(value.execute("SELECT max(id) FROM users").fetchone()[0])

    def restore(self, value: sqlite3.Connection, mark: int, /) -> None:
        value.execute("DELETE FROM users WHERE id > ?", (mark,))


def test_sign_up(name: str, names: list[str], db: sqlite3.Connection = use(UsersDb), outbox: Outbox = use(Outbox)) -> User:
    user = sign_up(db, outbox, name)
    assert user_names(db) == names
    assert outbox.sent == [(name, "welcome")]
    return user


ann = case(test_sign_up, name="ann", names=["old", "ann"])
bob = case(test_sign_up, name="bob", names=["old", "bob"])  # ann is not stored: each case has its own sample


def test_taken(name: str, names: list[str], db: sqlite3.Connection = use(SharedDb), outbox: Outbox = use(Outbox)) -> None:
    with raises(ValueError, match="taken"):
        sign_up(db, outbox, name)
    assert user_names(db) == names
    assert outbox.sent == []


old = case(test_taken, name="old", names=["old"])
