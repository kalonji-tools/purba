"""ex3: a stock (`use(UsersDb)`) and a fake (`use(outbox)`). Assays read the product or an input.

`taken` needs ann already stored: `first` is a name in the class body, so `after(first, "db")`.
"""

from sqlite3 import Connection

from app import Outbox, User, sign_up, user_names
from form_klasscall import Experiment, after, case, use
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

    first = case(act, name="ann").expect(user_id=1, stored=["ann"], mailed=[("ann", "welcome")])
    fresh = case(act, name="bob").expect(user_id=1, stored=["bob"], mailed=[("bob", "welcome")])
    taken = case(act, name="ann", db=after(first, "db")).raises(ValueError, "taken").expect(stored=["ann"], mailed=[])
