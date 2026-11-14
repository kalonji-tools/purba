"""ex4: a chain of three steps. `add` returns None, so its assays read the cart."""

from app import Cart, Receipt, open_cart
from form_record import Experiment, after


class Opened(Experiment[Cart]):
    owner: str

    def act(self) -> Cart:
        return open_cart(self.owner)

    def owner_seen(self) -> str:
        return self.product.owner

    def count(self) -> int:
        return self.product.count()


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


one_book  = Added(cart=after(ann), sku="BOOK-1", qty=1).expect(items=1, total=1200)
two_books = Added(cart=after(ann), sku="BOOK-1", qty=2).expect(items=2, total=2400)
pens      = Added(cart=after(ann), sku="PEN-7", qty=3).expect(items=3, total=300)  # 450
unknown   = Added(cart=after(ann), sku="NOPE", qty=1).raises(KeyError, "NOPE").expect(items=0, total=0)


class CheckedOut(Experiment[Receipt]):
    cart: Cart

    def act(self) -> Receipt:
        return self.cart.checkout()

    def paid(self) -> int:
        return self.product.total

    def lines(self) -> int:
        return self.product.lines


two_books_paid = CheckedOut(cart=after(two_books, "cart")).expect(paid=2400, lines=1)
pens_paid      = CheckedOut(cart=after(pens, "cart")).expect(paid=450, lines=1)
