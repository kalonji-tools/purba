"""ex6: what the parse refuses, before anything runs. Each case below is refused on its own; the others run."""

from app import slugify
from experiment import experiment


@experiment
def slug_of(text: str, sep: str = "-") -> str:
    return slugify(text, sep)


@slug_of.assay
def length(product: str) -> int:
    return len(product)


fine      = slug_of(text="a b").expect(product="a-b")
typo_in   = slug_of(txt="a b").expect(product="a-b")  # type: ignore
typo_out  = slug_of(text="a b").expect(lenght=3)
computed  = slug_of(text="a b").expect(product=slugify("a b"))
wrong     = slug_of(text="a b").expect(length="3")
float_for = slug_of(text="a b").expect(product=1)  # type: ignore
nothing   = slug_of(text="a b")
by_place  = slug_of("a b").expect(product="a-b")
again     = slug_of(text="x").expect(product="x")
again     = slug_of(text="y").expect(product="y")
slug_of(text="z").expect(product="z")
built     = [slug_of(text=t).expect(product=t) for t in "pq"]


@experiment
def branchy(text: str) -> str:
    if text:
        return slugify(text)
    return ""


some = branchy(text="a").expect(product="a")


@experiment
def looped(text: str) -> str:
    out = ""
    for _ in range(3):
        out = slugify(text)
    return out


thrice = looped(text="a").expect(product="a")
