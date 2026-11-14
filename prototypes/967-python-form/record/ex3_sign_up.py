"""ex3: a stock (`use(UsersDb)`) and a fake (`use(outbox)`) as fields. Assays read the product or a field.

`taken` needs ann already stored: a step of a chain, by its variable: `after(first, "db")`.
"""

from sqlite3 import Connection

from app import Outbox, User, sign_up, user_names
from form_record import Experiment, after, use
from lab import UsersDb, outbox


class SignedUp(Experiment[User]):
    name: str
    db: Connection = use(UsersDb)
    mailer: Outbox = use(outbox)

    def act(self) -> User:
        return sign_up(self.db, self.mailer, self.name)

    def user_id(self) -> int:
        return self.product.id

    def stored(self) -> list[str]:
        return user_names(self.db)

    def mailed(self) -> list[tuple[str, str]]:
        return self.mailer.sent


first = SignedUp(name="ann").expect(user_id=1, stored=["ann"], mailed=[("ann", "welcome")])
fresh = SignedUp(name="bob").expect(user_id=1, stored=["bob"], mailed=[("bob", "welcome")])
taken = SignedUp(name="ann", db=after(first, "db")).raises(ValueError, "taken").expect(stored=["ann"], mailed=[])
