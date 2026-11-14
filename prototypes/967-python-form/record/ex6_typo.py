"""ex6: a typo in the name of a step. Who catches it: purba's parse, Python's import, or the type checker?"""

from app import Cart, open_cart
from form_record import Experiment, after


class Opened(Experiment[Cart]):
    owner: str

    def act(self) -> Cart:
        return open_cart(self.owner)

    def count(self) -> int:
        return self.product.count()


ann = Opened(owner="ann").expect(count=0)


class Added(Experiment[None]):
    cart: Cart
    sku: str
    qty: int

    def act(self) -> None:
        return self.cart.add(self.sku, self.qty)

    def items(self) -> int:
        return self.cart.count()


one = Added(cart=after(an), sku="BOOK-1", qty=1).expect(items=1)  # "an": a typo


class Unrelated(Experiment[Cart]):
    owner: str

    def act(self) -> Cart:
        return open_cart(self.owner)

    def product_owner(self) -> str:
        return self.product.owner


bob = Unrelated(owner="bob").expect(product_owner="bob")
