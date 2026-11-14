"""ex4: a chain of three steps. `add` returns None, so checkout chains from the cart INPUT of a step."""

from typing import Self

from app import Cart, Receipt, open_cart
from form_record_strict import UNREAD, Experiment


class Opened(Experiment[Cart]):
    owner: str

    def act(self) -> Cart:
        return open_cart(self.owner)

    def owner_seen(self) -> str:
        return self.product.owner

    def count(self) -> int:
        return self.product.count()

    def expect(self, *, owner_seen: str = UNREAD, count: int = UNREAD) -> Self:
        return self


ann = Opened(owner="ann").expect(owner_seen="ann", count=0)


class Added(Experiment[None]):
    cart: Cart
    sku: str
    qty: int

    def act(self) -> None:
        return self.cart.add(self.sku, self.qty)

    def items(self) -> int:
        return self.cart.count()

    def total(self) -> int:
        return self.cart.total()

    def expect(self, *, items: int, total: int) -> Self:
        return self


one_book  = Added(cart=ann.product, sku="BOOK-1", qty=1).expect(items=1, total=1200)
two_books = Added(cart=ann.product, sku="BOOK-1", qty=2).expect(items=2, total=2400)
pens      = Added(cart=ann.product, sku="PEN-7", qty=3).expect(items=3, total=300)  # 450
unknown   = Added(cart=ann.product, sku="NOPE", qty=1).raises(KeyError, "NOPE").expect(items=0, total=0)


class CheckedOut(Experiment[Receipt]):
    cart: Cart

    def act(self) -> Receipt:
        return self.cart.checkout()

    def paid(self) -> int:
        return self.product.total

    def lines(self) -> int:
        return self.product.lines

    def expect(self, *, paid: int = UNREAD, lines: int = UNREAD) -> Self:
        return self


two_books_paid = CheckedOut(cart=two_books.cart).expect(paid=2400, lines=1)
pens_paid      = CheckedOut(cart=pens.cart).expect(paid=450, lines=1)
