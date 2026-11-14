from app import Cart, open_cart
from form_module import case


def act(owner: str) -> Cart:
    return open_cart(owner)


def count(product: Cart) -> int:
    return product.count()


ann = case(owner="ann", count=0)
