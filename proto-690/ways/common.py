"""Shared by every way. KEPT and saw() are instrumentation for run.py."""

from dataclasses import dataclass

import subject
from subject import Database

KEPT: list[int] = []  # ids of databases that went through the database builder
SEEN: list[tuple[Database, Database]] = []


def build_database() -> Database:
    """The body of the database builder, with its etiquette."""
    db = Database()
    KEPT.append(id(db))  # stands in for "opens a transaction it rolls back"
    return db


def saw(src: Database, dst: Database) -> None:
    SEEN.append((src, dst))


def copy(src: Database, dst: Database) -> None:
    subject.copy(src, dst)  # run.py swaps subject.copy for the planted bug


@dataclass(frozen=True)
class Seeded:
    key: str
