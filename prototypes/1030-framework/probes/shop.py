"""Code under test."""
RATE = 2


def price(n):
    if n < 0:
        raise ValueError("negative")
    return n * RATE
