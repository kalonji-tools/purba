"""The code under test: a copy from one database to another. Shared by every way."""


class Database:
    def __init__(self, name: str) -> None:
        self.name = name
        self.table: list[str] = []


class Tx[D: Database]:
    """A reagent: one transaction on a stock, rolled back after the test."""

    def __init__(self, db: D) -> None:
        self._db = db

    @property
    def db(self) -> D:
        return self._db

    def rows(self) -> list[str]:
        return list(self._db.table)

    def insert(self, row: str) -> None:
        self._db.table.append(row)


def copy[A: Database, B: Database](src: Tx[A], dst: Tx[B]) -> None:
    for row in src.rows():
        dst.insert(row)


def count() -> int:
    """A provider of the wrong type. The lie variants put it where a Tx belongs."""
    return 0
