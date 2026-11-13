"""PROTOTYPE (#963), throwaway. The "code under test" for the examples.

Plain app code, written the way people write it. Nothing here knows about
the experiment shape.
"""

import base64
import warnings
from dataclasses import dataclass, field
from decimal import ROUND_HALF_UP, Decimal
from typing import Protocol, runtime_checkable


# --- slugify ------------------------------------------------------------------
def slugify(text: str) -> str:
    words = text.strip().lower().split()
    if not words:
        raise ValueError("empty text")
    return "-".join(words)  # bug on purpose: accents are kept


def _normalise(text: str) -> str:  # a private helper
    return " ".join(text.split())


# --- money --------------------------------------------------------------------
@dataclass(frozen=True)
class Money:
    cents: int
    currency: str


def parse_amount(text: str, currency: str | None) -> Money:
    if currency is None:
        warnings.warn("parse_amount without a currency is deprecated", DeprecationWarning)
        currency = "USD"
    value = Decimal(text)
    if value < 0:
        raise ValueError(f"amount {text} is below zero")
    if value != value.quantize(Decimal("0.01")):
        warnings.warn(f"rounded {text} to cents", UserWarning)
    cents = int((value * 100).quantize(Decimal("1"), rounding=ROUND_HALF_UP))
    return Money(cents, currency)


# --- sign-up, with two collaborators ------------------------------------------
@runtime_checkable
class Store(Protocol):
    def add(self, name: str) -> int: ...
    def names(self) -> list[str]: ...


@runtime_checkable
class Mailer(Protocol):
    def send(self, to: str, subject: str) -> None: ...


@dataclass(frozen=True)
class User:
    id: int
    name: str


def sign_up(store: Store, mailer: Mailer, name: str) -> User:
    if name in store.names():
        raise ValueError(f"name {name!r} is taken")
    user = User(store.add(name), name)
    mailer.send(name, "welcome")
    return user


# --- a cart, with a mutating API (methods return None) ------------------------
PRICES = {"BOOK-1": 1200, "PEN-7": 150}


@dataclass(frozen=True)
class Receipt:
    owner: str
    total: int
    lines: int


@dataclass
class Cart:
    owner: str
    items: dict[str, int] = field(default_factory=dict)

    def add(self, sku: str, qty: int) -> None:
        if sku not in PRICES:
            raise KeyError(sku)
        self.items[sku] = self.items.get(sku, 0) + qty

    def count(self) -> int:
        return sum(self.items.values())

    def total(self) -> int:
        return sum(PRICES[s] * q for s, q in self.items.items())

    def checkout(self) -> Receipt:
        return Receipt(self.owner, self.total(), len(self.items))


def open_cart(owner: str) -> Cart:
    return Cart(owner)


# --- encoding -----------------------------------------------------------------
def encode(text: str) -> str:
    return base64.b64encode(text.encode()).decode()


def decode(blob: str) -> str:
    return base64.b64decode(blob).decode()


def sorted_copy(values: list[int]) -> list[int]:
    return sorted(values)
