from dataclasses import dataclass, field


class Database:
    def __init__(self, name: str) -> None:
        self.name = name
        self.table: list[str] = []


class Tx[D: Database]:
    """One transaction on a database. It is rolled back after each test."""

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


@dataclass(frozen=True)
class Settings:
    name: str


@dataclass(frozen=True)
class Clock:
    now: int


@dataclass
class Mailer:
    outbox: list[str] = field(default_factory=list)


class Client:
    def __init__(self, settings: Settings) -> None:
        self.settings = settings

    def fetch[D: Database](self, tx: Tx[D]) -> list[str]:
        return tx.rows()

    def signup[D: Database](self, tx: Tx[D], mailer: Mailer, clock: Clock, user: str) -> None:
        tx.insert(f"{user}@{clock.now}")
        mailer.outbox.append(f"welcome {user}")


@dataclass(frozen=True)
class Seeded:
    rows: int


def count() -> int:
    return 0
