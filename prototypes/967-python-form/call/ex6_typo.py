"""ex6: a typo in the name of a step. Who catches it: purba's parse, Python's import, or the type checker?"""

from app import Cart, open_cart
from form_call import after, experiment


@experiment
def opened(owner: str) -> Cart:
    return open_cart(owner)


@opened.assay
def count(product: Cart) -> int:
    return product.count()


ann = opened(owner="ann").expect(count=0)


@experiment
def added(cart: Cart, sku: str, qty: int) -> None:
    return cart.add(sku, qty)


@added.assay
def items(cart: Cart) -> int:
    return cart.count()


one = added(cart=after(an), sku="BOOK-1", qty=1).expect(items=1)  # "an": a typo


@experiment
def unrelated(owner: str) -> Cart:
    return open_cart(owner)


@unrelated.assay
def product_owner(product: Cart) -> str:
    return product.owner


bob = unrelated(owner="bob").expect(product_owner="bob")
