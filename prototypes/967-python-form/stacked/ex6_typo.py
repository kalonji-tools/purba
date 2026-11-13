"""ex6: a typo in the name of a step. Who catches it: purba's parse, Python's import, or the type checker?"""

from app import Cart, open_cart
from form_stacked import after, assay, case


@case("ann", input=dict(owner="ann"), assay=dict(count=0))
def opened(owner: str) -> Cart:
    return open_cart(owner)


@assay(opened)
def count(product: Cart) -> int:
    return product.count()


@case("one", input=dict(cart=after(opened, "an"), sku="BOOK-1", qty=1), assay=dict(items=1))  # "an": a typo
def added(cart: Cart, sku: str, qty: int) -> None:
    return cart.add(sku, qty)


@assay(added)
def items(cart: Cart) -> int:
    return cart.count()


@case("bob", input=dict(owner="bob"), assay=dict(product_owner="bob"))
def unrelated(owner: str) -> Cart:
    return open_cart(owner)


@assay(unrelated)
def product_owner(product: Cart) -> str:
    return product.owner
