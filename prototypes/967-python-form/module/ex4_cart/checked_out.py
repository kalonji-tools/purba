"""ex4, step 3 of 3."""

from added import pens, two_books
from app import Cart, Receipt
from form_module import after, case


def act(cart: Cart) -> Receipt:
    return cart.checkout()


def paid(product: Receipt) -> int:
    return product.total


def lines(product: Receipt) -> int:
    return product.lines


two_books_paid = case(cart=after(two_books, "cart"), paid=2400, lines=1)
pens_paid      = case(cart=after(pens, "cart"),      paid=450,  lines=1)
