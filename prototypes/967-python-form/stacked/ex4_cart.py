"""ex4: a chain of three steps. `add` returns None, so its assays read the cart."""

from app import Cart, Receipt, open_cart
from form_stacked import after, assay, case, raises


@case("ann", input=dict(owner="ann"), assay=dict(owner_seen="ann", count=0))
def opened(owner: str) -> Cart:
    return open_cart(owner)


@assay(opened)
def owner_seen(product: Cart) -> str:
    return product.owner


@assay(opened)
def count(product: Cart) -> int:
    return product.count()


@case("one_book",  input=dict(cart=after(opened, "ann"), sku="BOOK-1", qty=1), assay=dict(items=1, total=1200))
@case("two_books", input=dict(cart=after(opened, "ann"), sku="BOOK-1", qty=2), assay=dict(items=2, total=2400))
@case("pens",      input=dict(cart=after(opened, "ann"), sku="PEN-7", qty=3),  assay=dict(items=3, total=300))  # 450
@case("unknown",   input=dict(cart=after(opened, "ann"), sku="NOPE", qty=1),   assay=dict(items=0, total=0),
      raises=raises(KeyError, "NOPE"))
def added(cart: Cart, sku: str, qty: int) -> None:
    return cart.add(sku, qty)


@assay(added)
def items(cart: Cart) -> int:
    return cart.count()


@assay(added)
def total(cart: Cart) -> int:
    return cart.total()


@case("two_books", input=dict(cart=after(added, "two_books", "cart")), assay=dict(paid=2400, lines=1))
@case("pens",      input=dict(cart=after(added, "pens", "cart")),      assay=dict(paid=450, lines=1))
def checked_out(cart: Cart) -> Receipt:
    return cart.checkout()


@assay(checked_out)
def paid(product: Receipt) -> int:
    return product.total


@assay(checked_out)
def lines(product: Receipt) -> int:
    return product.lines
