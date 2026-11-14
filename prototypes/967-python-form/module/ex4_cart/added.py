"""ex4, step 2 of 3. `add` returns None, so its assays read the cart."""

from app import Cart
from form_module import after, case, raises
from opened import ann


def act(cart: Cart, sku: str, qty: int) -> None:
    return cart.add(sku, qty)


def items(cart: Cart) -> int:
    return cart.count()


def total(cart: Cart) -> int:
    return cart.total()


one_book  = case(cart=after(ann), sku="BOOK-1", qty=1, items=1, total=1200)
two_books = case(cart=after(ann), sku="BOOK-1", qty=2, items=2, total=2400)
pens      = case(cart=after(ann), sku="PEN-7",  qty=3, items=3, total=300)  # 450
unknown   = case(cart=after(ann), sku="NOPE",   qty=1, items=0, total=0, raises=raises(KeyError, "NOPE"))
