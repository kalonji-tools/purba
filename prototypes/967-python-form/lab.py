"""PROTOTYPE (#967). The fixtures every form shares. Never merge.

`outbox` is a plain fixture: a function that builds a value inside the process.
`UsersDb` is the fixture of a stock: its state lives outside the process, in a sqlite file.
It declares each offer as a base: a sample, a checkpoint and a fingerprint (#967 Q6).
"""

import os
import shutil
import sqlite3
import tempfile

from app import Outbox
from engine import Checkpoints, Fingerprints, Samples, Stock


def outbox() -> Outbox:
    return Outbox()


class UsersDb(Stock[sqlite3.Connection], Samples[sqlite3.Connection],
              Checkpoints[sqlite3.Connection, str], Fingerprints[sqlite3.Connection]):
    """The stock: one sqlite file that every case would share without a sample."""

    path = os.environ.get("PROTO967_DB", "")

    def open(self) -> sqlite3.Connection:
        db = sqlite3.connect(self.path, isolation_level=None)
        db.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT UNIQUE)")
        return db

    # Samples: draw a sample for one case; purba discards it after the case
    def sample(self, db: sqlite3.Connection) -> sqlite3.Connection:
        db.close()
        fd, copy = tempfile.mkstemp(suffix=".sqlite", prefix="PROTOTYPE-wipe-me-")
        os.close(fd)
        shutil.copyfile(self.path, copy)
        self._copy = copy
        return sqlite3.connect(copy, isolation_level=None)

    def discard(self, db: sqlite3.Connection) -> None:
        db.close()
        os.remove(self._copy)

    # Checkpoints: a point in the stock's state, and a return to it
    def checkpoint(self, db: sqlite3.Connection) -> str:
        db.execute("SAVEPOINT assay")
        return "assay"

    def restore(self, db: sqlite3.Connection, mark: str) -> None:
        db.execute(f"ROLLBACK TO {mark}")
        db.execute(f"RELEASE {mark}")

    # Fingerprints: purba compares it before and after the case
    def fingerprint(self, db: sqlite3.Connection) -> object:
        return db.execute("SELECT count(*), group_concat(name) FROM users").fetchone()
