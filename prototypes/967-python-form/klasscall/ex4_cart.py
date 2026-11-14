"""ex4: a chain of three steps. `add` returns None, so its assays read the cart."""

from app import Cart, Receipt, open_cart
from form_klasscall import Experiment, after, case


class Opened(Experiment):
    def act(owner: str) -> Cart:
        return open_cart(owner)

    def owner_seen(product: Cart) -> str:
        return product.owner

    def count(product: Cart) -> int:
        return product.count()

    ann = case(act, owner="ann").expect(owner_seen="ann", count=0)


class Added(Experiment):
    def act(cart: Cart, sku: str, qty: int) -> None:
        return cart.add(sku, qty)

    def items(cart: Cart) -> int:
        return cart.count()

    def total(cart: Cart) -> int:
        return cart.total()

    one_book  = case(act, cart=after(Opened.ann), sku="BOOK-1", qty=1).expect(items=1, total=1200)
    two_books = case(act, cart=after(Opened.ann), sku="BOOK-1", qty=2).expect(items=2, total=2400)
    pens      = case(act, cart=after(Opened.ann), sku="PEN-7", qty=3).expect(items=3, total=300)  # 450
    unknown   = case(act, cart=after(Opened.ann), sku="NOPE", qty=1).raises(KeyError, "NOPE").expect(items=0, total=0)


class CheckedOut(Experiment):
    def act(cart: Cart) -> Receipt:
        return cart.checkout()

    def paid(product: Receipt) -> int:
        return product.total

    def lines(product: Receipt) -> int:
        return product.lines

    two_books = case(act, cart=after(Added.two_books, "cart")).expect(paid=2400, lines=1)
    pens      = case(act, cart=after(Added.pens, "cart")).expect(paid=450, lines=1)
