"""ex4: a chain of three steps. `add` returns None, so its assays read the cart."""

from app import Cart, Receipt, open_cart
from form_below import after, experiment


@experiment
def opened(owner: str) -> Cart:
    return open_cart(owner)


@opened.assay
def owner_seen(product: Cart) -> str:
    return product.owner


@opened.assay
def count(product: Cart) -> int:
    return product.count()


ann = opened.case("ann", owner="ann").expect(owner_seen="ann", count=0)


@experiment
def added(cart: Cart, sku: str, qty: int) -> None:
    return cart.add(sku, qty)


@added.assay
def items(cart: Cart) -> int:
    return cart.count()


@added.assay
def total(cart: Cart) -> int:
    return cart.total()


added.case("one_book", cart=after(ann), sku="BOOK-1", qty=1).expect(items=1, total=1200)
two_books = added.case("two_books", cart=after(ann), sku="BOOK-1", qty=2).expect(items=2, total=2400)
pens = added.case("pens", cart=after(ann), sku="PEN-7", qty=3).expect(items=3, total=300)  # 450
added.case("unknown", cart=after(ann), sku="NOPE", qty=1).raises(KeyError, "NOPE").expect(items=0, total=0)


@experiment
def checked_out(cart: Cart) -> Receipt:
    return cart.checkout()


@checked_out.assay
def paid(product: Receipt) -> int:
    return product.total


@checked_out.assay
def lines(product: Receipt) -> int:
    return product.lines


checked_out.case("two_books", cart=after(two_books, "cart")).expect(paid=2400, lines=1)
checked_out.case("pens", cart=after(pens, "cart")).expect(paid=450, lines=1)
