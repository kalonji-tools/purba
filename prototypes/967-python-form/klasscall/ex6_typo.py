"""ex6: a typo in the name of a step. Who catches it: purba's parse, Python's import, or the type checker?"""

from app import Cart, open_cart
from form_klasscall import Experiment, after, case


class Opened(Experiment):
    def act(self, owner: str) -> Cart:
        return open_cart(owner)

    def count(self, product: Cart) -> int:
        return product.count()

    ann = case(act, owner="ann").expect(count=0)


class Added(Experiment):
    def act(self, cart: Cart, sku: str, qty: int) -> None:
        return cart.add(sku, qty)

    def items(self, cart: Cart) -> int:
        return cart.count()

    one = case(act, cart=after(Opened.an), sku="BOOK-1", qty=1).expect(items=1)  # "an": a typo


class Unrelated(Experiment):
    def act(self, owner: str) -> Cart:
        return open_cart(owner)

    def product_owner(self, product: Cart) -> str:
        return product.owner

    bob = case(act, owner="bob").expect(product_owner="bob")
