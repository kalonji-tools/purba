"""The code under test for `chains/test_web.py`: a small web app, its test client, and a session.

Each act appends a line to PIPE_LOG, to count how often a chain runs it. It sits outside the
test folder, so it is the code under test, not the act. APP_COST_MS makes `create_app` sleep
that long, to measure where the fork tree starts to save time (`timing.py cost`).
"""

from __future__ import annotations

import json
import os
import time
from collections.abc import Callable
from typing import Any
from dataclasses import dataclass, field


def _log(what: str) -> None:
    path = os.environ.get("PIPE_LOG")
    if path:
        with open(path, "a") as f:
            f.write(f"{os.getpid()} {what}\n")


@dataclass
class Response:
    status: int
    body: Any = None
    cookies: dict[str, str] = field(default_factory=dict)


Handler = Callable[["App", dict[str, object], str | None], Response]


class App:
    """What `create_app` builds: routes, users, orders, and a config."""

    def __init__(self, config: dict[str, object]) -> None:
        _log(f"create_app testing={config.get('TESTING')}")
        self.config = dict(config)
        self.users: dict[str, tuple[str, bool]] = {}  # name -> (password, admin)
        self.sessions: dict[str, str] = {}  # token -> name
        self.orders: list[dict[str, object]] = []
        self.prices = {"BOOK": 1200, "PEN": 200}

    def handle(
        self, method: str, path: str, body: dict[str, object], token: str | None
    ) -> Response:
        user = self.sessions.get(token or "")
        if (method, path) == ("GET", "/health"):
            return Response(200, {"ok": True, "testing": self.config.get("TESTING")})
        if (method, path) == ("POST", "/users"):
            name = str(body["name"])
            if name in self.users:
                return Response(409, {"error": "taken"})
            self.users[name] = (str(body["password"]), bool(body.get("admin", False)))
            return Response(201, {"name": name})
        if (method, path) == ("POST", "/login"):
            name = str(body["name"])
            if self.users.get(name, ("", False))[0] != body["password"]:
                return Response(401, {"error": "bad login"})
            token = f"t{len(self.sessions) + 1}-{name}"
            self.sessions[token] = name
            return Response(200, {"name": name}, {"session": token})
        if user is None:
            return Response(401, {"error": "log in first"})
        if (method, path) == ("GET", "/me"):
            return Response(200, {"name": user, "admin": self.users[user][1]})
        if (method, path) == ("GET", "/admin"):
            if not self.users[user][1]:
                return Response(403, {"error": "admins only"})
            return Response(200, {"users": sorted(self.users)})
        if (method, path) == ("POST", "/orders"):
            sku, qty = str(body["sku"]), int(str(body["qty"]))
            order = {
                "id": len(self.orders) + 1, "owner": user, "sku": sku, "qty": qty,
                "total": self.prices[sku] * qty, "state": "open",
            }
            self.orders.append(order)
            return Response(201, dict(order))
        if (method, path) == ("GET", "/orders"):
            return Response(200, [o["id"] for o in self.orders if o["owner"] == user])
        if method == "POST" and path.startswith("/orders/") and path.endswith("/cancel"):
            oid = int(path.split("/")[2])
            order = next((o for o in self.orders if o["id"] == oid and o["owner"] == user), None)
            if order is None:
                return Response(404, {"error": "no such order"})
            order["state"] = "cancelled"
            return Response(200, dict(order))
        if method == "GET" and path.startswith("/orders/"):
            oid = int(path.split("/")[2])
            order = next((o for o in self.orders if o["id"] == oid and o["owner"] == user), None)
            if order is None:
                return Response(404, {"error": "no such order"})
            return Response(200, dict(order))
        return Response(404, {"error": "no route"})


def create_app(testing: bool = True) -> App:
    time.sleep(float(os.environ.get("APP_COST_MS", "0")) / 1000)
    return App({"TESTING": testing, "SECRET": "s3cret"})


class Client:
    """A test client of one app: it keeps a session cookie, as a browser does."""

    def __init__(self, app: App) -> None:
        _log("client")
        self.app = app
        self.token: str | None = None

    def request(self, method: str, path: str, body: dict[str, object] | None = None) -> Response:
        _log(f"{method} {path}")
        response = self.app.handle(method, path, json.loads(json.dumps(body or {})), self.token)
        if "session" in response.cookies:
            self.token = response.cookies["session"]
        return response

    def get(self, path: str) -> Response:
        return self.request("GET", path)

    def post(self, path: str, body: dict[str, object] | None = None) -> Response:
        return self.request("POST", path, body)
