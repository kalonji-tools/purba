"""#1010: a pytest suite whose fixtures depend on other fixtures, as a chain of stages.

In pytest this is `app` → `client` → `seeded` → `ann_client` → a request. Each fixture
builds on the one before. Here each fixture is a stage that checks its own state and hands
its product on.
"""

from __future__ import annotations

from dataclasses import dataclass

from cases import case
from webapp import App, Client, create_app


# ── app → client: two roots, as a parametrised `app` fixture ──────────


def test_app(testing: bool) -> App:
    app = create_app(testing=testing)
    assert app.config["TESTING"] == testing
    return app


app = case(test_app, testing=True)
prod_app = case(test_app, testing=False)


def test_client(app: App) -> Client:
    client = Client(app)
    health = client.get("/health")
    assert health.status == 200
    return client


client = case(test_client, app=app.product)
prod_client = case(test_client, app=prod_app.product)


def test_health_body(client: Client, testing: bool) -> None:
    body = client.get("/health").body
    assert body == {"ok": True, "testing": testing}


health = case(test_health_body, client=client.product, testing=True)
prod_health = case(test_health_body, client=prod_client.product, testing=False)


# ── seeded: the users every later test logs in as ───────────────


def test_seed(client: Client) -> App:
    ann = client.post("/users", {"name": "ann", "password": "pw-ann"})
    bob = client.post("/users", {"name": "bob", "password": "pw-bob"})
    root = client.post("/users", {"name": "root", "password": "pw-root", "admin": True})
    assert [ann.status, bob.status, root.status] == [201, 201, 201]
    return client.app


seeded = case(test_seed, client=client.product)


def test_sign_up_taken(app: App, name: str) -> None:
    again = Client(app).post("/users", {"name": name, "password": "x"})
    assert again.status == 409


ann_taken = case(test_sign_up_taken, app=seeded.product, name="ann")
root_taken = case(test_sign_up_taken, app=seeded.product, name="root")


# ── a logged-in client for each user, as `ann_client`, `bob_client`, `admin_client` ──


@dataclass
class Session:
    client: Client
    name: str


def test_login(app: App, name: str, password: str) -> Session:
    client = Client(app)
    response = client.post("/login", {"name": name, "password": password})
    assert response.status == 200
    return Session(client, name)


ann_in = case(test_login, app=seeded.product, name="ann", password="pw-ann")
bob_in = case(test_login, app=seeded.product, name="bob", password="pw-bob")
root_in = case(test_login, app=seeded.product, name="root", password="pw-root")


def test_bad_login(app: App, name: str, password: str) -> None:
    response = Client(app).post("/login", {"name": name, "password": password})
    assert response.status == 401


bad_password = case(test_bad_login, app=seeded.product, name="ann", password="nope")
no_such_user = case(test_bad_login, app=seeded.product, name="eve", password="pw-eve")


# ── requests: the tests at the end of each fixture chain ────────────


def test_get(session: Session, path: str, status: int) -> None:
    response = session.client.get(path)
    assert response.status == status


ann_me = case(test_get, session=ann_in.product, path="/me", status=200)
ann_admin = case(test_get, session=ann_in.product, path="/admin", status=403)
bob_me = case(test_get, session=bob_in.product, path="/me", status=200)
root_admin = case(test_get, session=root_in.product, path="/admin", status=200)
root_nowhere = case(test_get, session=root_in.product, path="/nowhere", status=404)


def test_me(session: Session, name: str, admin: bool) -> None:
    body = session.client.get("/me").body
    assert body == {"name": name, "admin": admin}


ann_body = case(test_me, session=ann_in.product, name="ann", admin=False)
root_body = case(test_me, session=root_in.product, name="root", admin=True)


def test_orders_listed(session: Session, ids: list[int]) -> None:
    listed = session.client.get("/orders").body
    assert listed == ids


ann_no_orders = case(test_orders_listed, session=ann_in.product, ids=[])


# ── an order, and what follows it: a chain six stages deep ────────────


@dataclass
class Placed:
    session: Session
    order: int


def test_order(session: Session, sku: str, qty: int, total: int) -> Placed:
    response = session.client.post("/orders", {"sku": sku, "qty": qty})
    assert response.status == 201
    assert response.body["total"] == total
    return Placed(session, response.body["id"])


ann_book = case(test_order, session=ann_in.product, sku="BOOK", qty=2, total=2400)
ann_pen = case(test_order, session=ann_in.product, sku="PEN", qty=1, total=200)
bob_pen = case(test_order, session=bob_in.product, sku="PEN", qty=5, total=1000)


def test_read_order(placed: Placed, state: str) -> None:
    body = placed.session.client.get(f"/orders/{placed.order}").body
    assert body["state"] == state


ann_book_open = case(test_read_order, placed=ann_book.product, state="open")
bob_pen_open = case(test_read_order, placed=bob_pen.product, state="open")


def test_listed_after(placed: Placed, ids: list[int]) -> None:
    listed = placed.session.client.get("/orders").body
    assert listed == ids


ann_lists_book = case(test_listed_after, placed=ann_book.product, ids=[1])


def test_cancel(placed: Placed) -> Placed:
    response = placed.session.client.post(f"/orders/{placed.order}/cancel")
    assert response.body["state"] == "cancelled"
    return placed


ann_book_cancelled = case(test_cancel, placed=ann_book.product)
bob_pen_cancelled = case(test_cancel, placed=bob_pen.product)
cancelled_read = case(test_read_order, placed=ann_book_cancelled.product, state="cancelled")


# ── fan-in: a test that takes two fixtures of one chain ─────────────


def test_cannot_cancel_other(owner: Placed, other: Session) -> None:
    response = other.client.post(f"/orders/{owner.order}/cancel")
    assert response.status == 404


bob_cannot_cancel = case(test_cannot_cancel_other, owner=ann_book.product, other=bob_in.product)


# ── a stage that fails, and one that is refused: each blocks what follows ───────

wrong_total = case(test_order, session=bob_in.product, sku="BOOK", qty=1, total=999)
after_wrong = case(test_read_order, placed=wrong_total.product, state="open")


def test_login_one_way(app: App, name: str, password: str) -> Session:
    client = Client(app)
    response = client.post("/login", {"name": name, "password": password})
    token = client.token if response.status == 200 else ""
    assert token != ""
    return Session(client, name)


one_way_in = case(test_login_one_way, app=seeded.product, name="bob", password="pw-bob")
after_one_way = case(test_get, session=one_way_in.product, path="/me", status=200)
