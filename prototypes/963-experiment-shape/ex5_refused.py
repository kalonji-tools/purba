"""ex5: what the allow list refuses before anything runs, and a departure with a reason."""

from experiment import REGISTRY, builder, experiment
from prototype_app import Mailer, _normalise, slugify


@experiment(columns=("text", "slug"), a=("Hi", "hi"))
def branchy(text: str) -> str:
    if not text:
        return ""
    return slugify(text)


@experiment(columns=("text", "slug"), a=("Hi", "hi"))
def two_lines(text: str) -> str:
    cleaned = text.strip()
    return slugify(cleaned)


@experiment(columns=("text", "slug"), a=("Hi", "hi"))
def nested(text: str) -> str:
    return slugify(text.strip())


@experiment(columns=("text", "slug"), a=("Hi", "hi"))
def private_act(text: str) -> str:
    return _normalise(text)


@experiment(
    columns=("text", "slug"),
    a=("a  b", "a b"),
    allow={"private-name": "PEP 399: the C and the pure version must agree"},
)
def private_with_reason(text: str) -> str:
    return _normalise(text)


@experiment(columns=("text",), a=("Hi",))
def no_assay(text: str) -> str:
    return slugify(text)


@experiment(columns=("text", "slgu"), a=("Hi", "hi"))
def typo_column(text: str) -> str:
    return slugify(text)


@experiment(columns=("text", "slug"), a=("Hi", "hi"))
def slug_twice(text: str) -> str:
    return slugify(text)


@experiment(columns=("text", "slug"), a=("Hi", "hi"))
def slug_twice(text: str) -> str:  # noqa: F811
    return slugify(text)


def same(product: str) -> str:
    return product


# each experiment above gets the same `slug` assay; the refusals are the point
NAMES = {"branchy", "two_lines", "nested", "private_act", "private_with_reason", "typo_column", "slug_twice"}
for e in REGISTRY:
    if e.name in NAMES:
        e.assays["slug"] = same


@experiment(columns=("text", "slug"), a=("Hi", "hi"))
def logic_in_assay(text: str) -> str:
    return slugify(text)


@logic_in_assay.assay
def slug(product: str) -> str:
    return product or "empty"


# a stand-in with a typo: refused when it is built, before the act runs
class TypoMailer:
    def sned(self, to, subject):
        pass


@builder
def mailer(seed: tuple) -> Mailer:
    return TypoMailer()


@experiment(columns=("mailer", "to", "subject", "sent"), a=((), "ann", "hi", None))
def mailed_typo(mailer: Mailer, to: str, subject: str) -> None:
    return mailer.send(to, subject)


@mailed_typo.assay
def sent(product: None) -> None:
    return product
