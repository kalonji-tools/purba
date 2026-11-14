"""ex6: a typo in the name of a step. Who catches it: purba's parse, Python's import, or the type checker?"""

from typing import Self

from app import Cart, open_cart
from form_record_strict import UNREAD, Experiment


class Opened(Experiment[Cart]):
    owner: str

    def act(self) -> Cart:
        return open_cart(self.owner)

    def count(self) -> int:
        return self.product.count()

    def expect(self, *, count: int = UNREAD) -> Self:
        return self


ann = Opened(owner="ann").expect(count=0)


class Added(Experiment[None]):
    cart: Cart
    sku: str
    qty: int

    def act(self) -> None:
        return self.cart.add(self.sku, self.qty)

    def items(self) -> int:
        return self.cart.count()

    def expect(self, *, items: int) -> Self:
        return self


one = Added(cart=an.product, sku="BOOK-1", qty=1).expect(items=1)  # "an": a typo
