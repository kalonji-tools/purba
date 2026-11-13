"""ex6: a typo in the name of a step. Who catches it: purba's parse, Python's import, or the type checker?"""

from app import Cart, open_cart
from form_klass import Experiment, after, case


class Opened(Experiment):
    def act(owner: str) -> Cart:
        return open_cart(owner)

    def count(product: Cart) -> int:
        return product.count()

    ann = case(owner="ann", count=0)


class Added(Experiment):
    def act(cart: Cart, sku: str, qty: int) -> None:
        return cart.add(sku, qty)

    def items(cart: Cart) -> int:
        return cart.count()

    one = case(cart=after(Opened.an), sku="BOOK-1", qty=1, items=1)  # "an": a typo


class Unrelated(Experiment):
    def act(owner: str) -> Cart:
        return open_cart(owner)

    def product_owner(product: Cart) -> str:
        return product.owner

    bob = case(owner="bob", product_owner="bob")
