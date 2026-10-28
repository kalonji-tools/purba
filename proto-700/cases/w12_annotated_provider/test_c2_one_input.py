from typing import Annotated

from fixtures import database
from sketch import From
from subject import Database, Tx


def test_insert(db: Annotated[Tx[Database], From(database)]) -> None:
    db.insert("a")
    assert db.rows() == ["a"], "the row is in the table"
