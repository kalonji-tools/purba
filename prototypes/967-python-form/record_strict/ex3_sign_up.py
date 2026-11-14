"""ex3: a stock and a fake as fields. `taken` chains from an INPUT of `first`: `db=first.db`, typed."""

from sqlite3 import Connection
from typing import Self

from app import Outbox, User, sign_up, user_names
from form_record_strict import UNREAD, Experiment, use
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

    def expect(self, *, stored: list[str], mailed: list[tuple[str, str]], user_id: int = UNREAD) -> Self:
        return self


first = SignedUp(name="ann").expect(user_id=1, stored=["ann"], mailed=[("ann", "welcome")])
fresh = SignedUp(name="bob").expect(user_id=1, stored=["bob"], mailed=[("bob", "welcome")])
taken = SignedUp(name="ann", db=first.db).raises(ValueError, "taken").expect(stored=["ann"], mailed=[])
