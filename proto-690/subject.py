"""The code under test: a copy from one database to another.

`copy` is swapped for `copy_nothing` by run.py to plant a bug. A way of
naming two inputs works only if the planted bug FAILS the test.
"""


class Database:
    def __init__(self) -> None:
        self.rows: dict[str, int] = {}

    def insert(self, key: str, value: int) -> None:
        self.rows[key] = value


def copy(src: Database, dst: Database) -> None:
    dst.rows.update(src.rows)


def copy_nothing(src: Database, dst: Database) -> None:
    """The planted bug: copies nothing."""
