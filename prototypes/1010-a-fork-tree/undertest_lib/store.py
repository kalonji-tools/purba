"""The code under test for `chains/test_store.py`: a user table in a SQLite file.

Each act appends to PIPE_LOG.
"""

from __future__ import annotations

import os
import sqlite3
from pathlib import Path


def _log(what: str) -> None:
    path = os.environ.get("PIPE_LOG")
    if path:
        with open(path, "a") as f:
            f.write(f"{os.getpid()} {what}\n")


def migrate(db: Path) -> None:
    _log("migrate")
    with sqlite3.connect(db) as conn:
        conn.execute("create table users (name text primary key)")
    conn.close()


def add_user(db: Path, name: str) -> None:
    _log(f"add_user {name}")
    with sqlite3.connect(db) as conn:
        conn.execute("insert into users values (?)", (name,))
    conn.close()


def tables(db: Path) -> list[str]:
    with sqlite3.connect(db) as conn:
        query = "select name from sqlite_master where type = 'table' order by name"
        found = [row[0] for row in conn.execute(query)]
    conn.close()
    return found


def users(db: Path) -> list[str]:
    with sqlite3.connect(db) as conn:
        found = [row[0] for row in conn.execute("select name from users order by rowid")]
    conn.close()
    return found
