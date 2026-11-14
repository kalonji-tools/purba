"""The code under test, copied from prototype 1 (#967). Never merge."""

import sqlite3
import unicodedata
import warnings
from dataclasses import dataclass


def slugify(text: str, sep: str = "-") -> str:
    words = text.split()
    if not words:
        raise ValueError("empty")
    return sep.join(w.lower() for w in words)  # keeps accents: a bug the tests find


def _normalise(text: str) -> str:
    return unicodedata.normalize("NFKD", text)


@dataclass(frozen=True)
class Money:
    cents: int
    currency: str


def parse_amount(text: str, currency: str | None) -> Money:
    if currency is None:
        warnings.warn("no currency: USD is assumed", DeprecationWarning, stacklevel=2)
        currency = "USD"
    value = float(text)
    if value < 0:
        raise ValueError("below zero")
    cents = round(value * 100)
    if abs(value * 100 - cents) > 1e-9:
        warnings.warn("rounded to a cent", UserWarning, stacklevel=2)
    return Money(cents, currency)


@dataclass(frozen=True)
class User:
    id: int
    name: str


class Outbox:
    """A fake mailer: it keeps what it would send."""

    def __init__(self) -> None:
        self.sent: list[tuple[str, str]] = []

    def send(self, to: str, subject: str) -> None:
        self.sent.append((to, subject))


def sign_up(db: sqlite3.Connection, outbox: Outbox, name: str) -> User:
    if db.execute("SELECT 1 FROM users WHERE name = ?", (name,)).fetchone():
        raise ValueError(f"name {name!r} is taken")
    cur = db.execute("INSERT INTO users (name) VALUES (?)", (name,))
    outbox.send(name, "welcome")
    return User(int(cur.lastrowid or 0), name)


PRICES = {"BOOK-1": 1200, "PEN-7": 150}


@dataclass(frozen=True)
class Receipt:
    total: int
    lines: int


class Cart:
    def __init__(self, owner: str) -> None:
        self.owner = owner
        self.items: dict[str, int] = {}

    def add(self, sku: str, qty: int) -> None:
        if sku not in PRICES:
            raise KeyError(sku)
        self.items[sku] = self.items.get(sku, 0) + qty

    def count(self) -> int:
        return sum(self.items.values())

    def total(self) -> int:
        return sum(PRICES[s] * q for s, q in self.items.items())

    def checkout(self) -> Receipt:
        return Receipt(self.total(), len(self.items))


def open_cart(owner: str) -> Cart:
    return Cart(owner)


def user_names(db: sqlite3.Connection) -> list[str]:
    return [n for (n,) in db.execute("SELECT name FROM users ORDER BY id")]


def check_age(age: int) -> int:
    if age < 18:
        raise ValueError("too young")
    return age


def adult_count(ages: list[int]) -> int:
    return sum(a >= 18 for a in ages) if len(ages) > 1 else ages[0] >= 18  # a bug: one age gives a bool
