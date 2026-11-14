"""ex3: a stock (`use(UsersDb)`) and a fake (`use(outbox)`). Assays read the product or an input.

`taken` needs ann already stored: a step of a chain, by its variable: `after(first, "db")`.
"""

from sqlite3 import Connection

from app import Outbox, User, sign_up, user_names
from form_call import after, experiment, use
from lab import UsersDb, outbox


@experiment
def signed_up(name: str, db: Connection = use(UsersDb), mailer: Outbox = use(outbox)) -> User:
    return sign_up(db, mailer, name)


@signed_up.assay
def user_id(product: User) -> int:
    return product.id


@signed_up.assay
def stored(db: Connection) -> list[str]:
    return user_names(db)


@signed_up.assay
def mailed(mailer: Outbox) -> list[tuple[str, str]]:
    return mailer.sent


first = signed_up(name="ann").expect(user_id=1, stored=["ann"], mailed=[("ann", "welcome")])
fresh = signed_up(name="bob").expect(user_id=1, stored=["bob"], mailed=[("bob", "welcome")])
taken = signed_up(name="ann", db=after(first, "db")).raises(ValueError, "taken").expect(stored=["ann"], mailed=[])
