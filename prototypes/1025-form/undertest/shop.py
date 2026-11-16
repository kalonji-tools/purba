"""The code under test: a shop whose business logic is a pipeline. Each act appends to PIPE_LOG, to
count replays."""

from __future__ import annotations

import os
from dataclasses import dataclass, field


def _log(what: str) -> None:
    path = os.environ.get("PIPE_LOG")
    if path:
        with open(path, "a") as f:
            f.write(f"{os.getpid()} {what}\n")


@dataclass
class Order:
    owner: str
    total: int
    state: str = "paid"


@dataclass
class Shop:
    users: list[str] = field(default_factory=list)
    carts: dict[str, list[tuple[str, int]]] = field(default_factory=dict)
    orders: list[Order] = field(default_factory=list)
    prices: dict[str, int] = field(default_factory=lambda: {"BOOK": 1200, "PEN": 200})

    def sign_up(self, name: str) -> str:
        _log(f"sign_up {name}")
        if name in self.users:
            raise ValueError(f"{name} is taken")
        self.users.append(name)
        self.carts[name] = []
        return name

    def add(self, user: str, sku: str, qty: int) -> int:
        _log(f"add {user} {sku}")
        self.carts[user].append((sku, qty))
        return len(self.carts[user])

    def checkout(self, user: str) -> Order:
        _log(f"checkout {user}")
        total = sum(self.prices[sku] * qty for sku, qty in self.carts[user])
        order = Order(user, total)
        self.orders.append(order)
        self.carts[user] = []
        return order

    def refund(self, order: Order) -> Order:
        _log(f"refund {order.owner}")
        order.state = "refunded"
        return order
