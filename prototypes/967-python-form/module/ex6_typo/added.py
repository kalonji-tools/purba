"""ex6: a typo in the name of a step. Who catches it: purba's parse, Python's import, or the type checker?"""

from app import Cart
from form_module import after, case
from opened import an  # "an": a typo


def act(cart: Cart, sku: str, qty: int) -> None:
    return cart.add(sku, qty)


def items(cart: Cart) -> int:
    return cart.count()


one = case(cart=after(an), sku="BOOK-1", qty=1, items=1)
