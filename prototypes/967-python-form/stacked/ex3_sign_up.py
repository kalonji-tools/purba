"""ex3: a stock (`use(UsersDb)`) and a fake (`use(outbox)`). Assays read the product or an input.

`taken` needs ann already stored: a step of a chain. It names its own experiment, which does not
exist yet while its decorators run, so it names it by TEXT: `after("signed_up", ...)`.
"""

from sqlite3 import Connection

from app import Outbox, User, sign_up, user_names
from form_stacked import after, assay, case, raises, use
from lab import UsersDb, outbox


@case("first", input=dict(name="ann"), assay=dict(user_id=1, stored=["ann"], mailed=[("ann", "welcome")]))
@case("fresh", input=dict(name="bob"), assay=dict(user_id=1, stored=["bob"], mailed=[("bob", "welcome")]))
@case("taken", input=dict(name="ann", db=after("signed_up", "first", "db")), assay=dict(stored=["ann"], mailed=[]),
      raises=raises(ValueError, "taken"))
def signed_up(name: str, db: Connection = use(UsersDb), mailer: Outbox = use(outbox)) -> User:
    return sign_up(db, mailer, name)


@assay(signed_up)
def user_id(product: User) -> int:
    return product.id


@assay(signed_up)
def stored(db: Connection) -> list[str]:
    return user_names(db)


@assay(signed_up)
def mailed(mailer: Outbox) -> list[tuple[str, str]]:
    return mailer.sent
