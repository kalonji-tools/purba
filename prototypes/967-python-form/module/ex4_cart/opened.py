"""ex4, step 1 of 3."""

from app import Cart, open_cart
from form_module import case


def act(owner: str) -> Cart:
    return open_cart(owner)


def owner_seen(product: Cart) -> str:
    return product.owner


def count(product: Cart) -> int:
    return product.count()


ann = case(owner="ann", owner_seen="ann", count=0)
