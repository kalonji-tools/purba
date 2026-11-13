"""ex4: a process of several steps = a chain of experiments. `add` returns None, so its assays read the cart."""

from experiment import after, experiment, raises
from prototype_app import Cart, Receipt, open_cart


@experiment(
    columns=("owner", "owner_seen", "count"),
    ann=    ("ann",   "ann",        0),
)
def opened(owner: str) -> Cart:
    return open_cart(owner)


@opened.assay
def owner_seen(product: Cart) -> str:
    return product.owner


@opened.assay
def count(product: Cart) -> int:
    return product.count()


@experiment(
    columns=  ("cart",               "sku",    "qty", "items", "total", "raises"),
    one_book= (after(opened, "ann"), "BOOK-1", 1,     1,       1200,    ...),
    two_books=(after(opened, "ann"), "BOOK-1", 2,     2,       2400,    ...),
    pens=     (after(opened, "ann"), "PEN-7",  3,     3,       300,     ...),  # wrong on purpose: 3 pens = 450
    unknown=  (after(opened, "ann"), "NOPE",   1,     0,       0,       raises(KeyError, "NOPE")),
)
def added(cart: Cart, sku: str, qty: int) -> None:
    return cart.add(sku, qty)


@added.assay
def items(cart: Cart) -> int:
    return cart.count()


@added.assay
def total(cart: Cart) -> int:
    return cart.total()


@experiment(
    columns=  ("cart",                           "paid", "lines"),
    two_books=(after(added, "two_books", "cart"), 2400,   1),
    pens=     (after(added, "pens", "cart"),      450,    1),  # its step failed → blocked
)
def checked_out(cart: Cart) -> Receipt:
    return cart.checkout()


@checked_out.assay
def paid(product: Receipt) -> int:
    return product.total


@checked_out.assay
def lines(product: Receipt) -> int:
    return product.lines
