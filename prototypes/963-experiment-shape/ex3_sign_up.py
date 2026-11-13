"""ex3: stand-ins enter only as inputs, built per case. Assays read the product OR an input after the act."""

from experiment import builder, experiment, raises
from prototype_app import Mailer, Store, User, sign_up


class InMemoryStore:
    def __init__(self, names):
        self.rows = list(names)

    def add(self, name):
        self.rows.append(name)
        return len(self.rows)

    def names(self):
        return list(self.rows)


class OutboxMailer:
    def __init__(self):
        self.outbox = []

    def send(self, to, subject):
        self.outbox.append((to, subject))


@builder
def store(names: tuple[str, ...]) -> Store:
    return InMemoryStore(names)


@builder
def mailer(seed: tuple) -> Mailer:
    return OutboxMailer()


@experiment(
    columns=("store",   "mailer", "name", "user_id", "stored",        "mailed",              "raises"),
    first=  ((),        (),       "ann",  1,         ["ann"],         [("ann", "welcome")],  ...),
    second= (("bob",),  (),       "ann",  2,         ["bob", "ann"],  [("ann", "welcome")],  ...),
    taken=  (("ann",),  (),       "ann",  ...,       ["ann"],         [],                    raises(ValueError, "taken")),
)
def signed_up(store: Store, mailer: Mailer, name: str) -> User:
    return sign_up(store, mailer, name)


@signed_up.assay
def user_id(product: User) -> int:
    return product.id


@signed_up.assay
def stored(store: InMemoryStore) -> list[str]:
    return store.names()


@signed_up.assay
def mailed(mailer: OutboxMailer) -> list:
    return mailer.outbox
