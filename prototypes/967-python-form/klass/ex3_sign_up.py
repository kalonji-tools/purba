"""ex3: a stock (`use(UsersDb)`) and a fake (`use(outbox)`). Assays read the product or an input.

`taken` needs ann already stored: a step of a chain. `first` is a name in the class body, so `after(first, "db")`.
"""

from sqlite3 import Connection

from app import Outbox, User, sign_up, user_names
from form_klass import Experiment, after, case, raises, use
from lab import UsersDb, outbox


class SignedUp(Experiment):
    def act(name: str, db: Connection = use(UsersDb), mailer: Outbox = use(outbox)) -> User:
        return sign_up(db, mailer, name)

    def user_id(product: User) -> int:
        return product.id

    def stored(db: Connection) -> list[str]:
        return user_names(db)

    def mailed(mailer: Outbox) -> list[tuple[str, str]]:
        return mailer.sent

    first = case(name="ann", user_id=1, stored=["ann"], mailed=[("ann", "welcome")])
    fresh = case(name="bob", user_id=1, stored=["bob"], mailed=[("bob", "welcome")])
    taken = case(name="ann", db=after(first, "db"), stored=["ann"], mailed=[], raises=raises(ValueError, "taken"))
