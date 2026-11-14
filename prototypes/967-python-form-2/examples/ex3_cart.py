"""ex3: a stateful API in one act, and a chain that takes an earlier case's product."""

from app import Cart, Receipt, open_cart
from experiment import experiment


@experiment
def bought(owner: str, sku: str, qty: int) -> Receipt:
    cart = open_cart(owner)
    cart.add(sku, qty)
    cart.add("PEN-7", 1)
    return cart.checkout()


@bought.assay
def paid(product: Receipt) -> int:
    """the total of the receipt, in cents"""
    return product.total


@bought.assay
def lines(product: Receipt) -> int:
    return product.lines


two_books = bought(owner="ann", sku="BOOK-1", qty=2).expect(paid=2550, lines=2)
pens      = bought(owner="ann", sku="PEN-7", qty=3).expect(paid=600, lines=1)
unknown   = bought(owner="ann", sku="NOPE", qty=1).raises(KeyError, match="NOPE")


# A chain: a later case starts from what an earlier case made. The earlier case runs again first, once.
@experiment
def opened(owner: str) -> Cart:
    return open_cart(owner)


@opened.assay
def count(product: Cart) -> int:
    return product.count()


ann = opened(owner="ann").expect(count=0)


@experiment
def filled(cart: Cart, again: Cart) -> int:
    cart.add("BOOK-1", 1)
    again.add("BOOK-1", 1)
    return cart.total()


both_inputs_one_cart = filled(cart=ann.product, again=ann.product).expect(product=2400)

bob        = opened(owner="bob").expect(count=1)  # a wrong expected value, so the chain below is blocked
from_wrong = filled(cart=bob.product, again=bob.product).expect(product=2400)
