"""#1010 P8: one test, run alone on its fixture, or as a stage on a part of an earlier product."""

from __future__ import annotations

from cases import case, use
from shop import Shop
from test_pipeline import ann, shop_with_ann


def test_add_one(shop: Shop = use(shop_with_ann)) -> None:
    shop.add("ann", "PEN", 1)
    assert shop.carts["ann"] == [("PEN", 1)]


alone    = case(test_add_one)
as_stage = case(test_add_one, shop=ann.product.shop)
