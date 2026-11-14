from app import Cart, open_cart
from form_module import case


def act(owner: str) -> Cart:
    return open_cart(owner)


def product_owner(product: Cart) -> str:
    return product.owner


bob = case(owner="bob", product_owner="bob")
