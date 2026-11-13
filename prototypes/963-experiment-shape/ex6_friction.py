"""ex6: where the shape hurts. Each passes or fails for a reason worth seeing."""

import os

from experiment import builder, experiment
from prototype_app import decode, encode, sorted_copy


# 1. A rule, not a value: "decode(encode(x)) gives x back".
#    The act may not nest calls, so the logic moves into a helper the parser never reads,
#    and the table repeats the input as the expected value. "For EVERY x" can't be said.
def roundtrip(text: str) -> str:
    return decode(encode(text))


@experiment(
    columns=("text",  "back"),
    ascii=  ("hello", "hello"),
    accent= ("héllo", "héllo"),
)
def round_tripped(text: str) -> str:
    return roundtrip(text)


@round_tripped.assay
def back(product: str) -> str:
    return product


# 2. A branch hidden in a builder: the parser never looks there. This passes on a stub.
class FakeDb:
    def ping(self):
        return "pong"


@builder
def db(url: str):
    if not os.environ.get("REAL_DB"):  # ← the branch moved out of the test
        return FakeDb()
    raise RuntimeError("would connect to " + url)


def ping(db) -> str:
    return db.ping()


@experiment(columns=("db", "pong"), local=("postgres://prod", "pong"))
def pinged(db) -> str:
    return ping(db)


@pinged.assay
def pong(product: str) -> str:
    return product


# 3. Sealing by copying: each assay gets its own copy of a large product.
@experiment(
    columns=("values",                   "first", "last",    "size"),
    big=    (list(range(300_000, 0, -1)), 1,       300_000,   300_000),
)
def sorted_big(values: list[int]) -> list[int]:
    return sorted_copy(values)


@sorted_big.assay
def first(product: list[int]) -> int:
    return product[0]


@sorted_big.assay
def last(product: list[int]) -> int:
    return product[-1]


@sorted_big.assay
def size(product: list[int]) -> int:
    return len(product)
