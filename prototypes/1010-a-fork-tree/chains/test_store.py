"""#1010: a database fixture (`tmp_path` → `db` → `db_with_ann`).

Its state lives in a file. It is a stock, which a fork does not copy: there the fork tree
must fall back to replay.
"""

from __future__ import annotations

import hashlib
import shutil
import tempfile
from pathlib import Path

from cases import Fingerprints, Samples, case, use
from store import add_user, migrate, tables, users


class Db(Samples[Path], Fingerprints[Path]):
    """A SQLite file in a folder of its own for each case, discarded after it."""

    def sample(self) -> Path:
        return Path(tempfile.mkdtemp(prefix="p1010-db-")) / "app.db"

    def discard(self, value: Path, /) -> None:
        shutil.rmtree(value.parent)

    def fingerprint(self, value: Path, /) -> object:
        return hashlib.sha256(value.read_bytes()).hexdigest() if value.exists() else None


def test_schema(db: Path = use(Db)) -> Path:
    migrate(db)
    assert tables(db) == ["users"]
    return db


schema = case(test_schema)


def test_add(db: Path, name: str, names: list[str]) -> Path:
    add_user(db, name)
    assert users(db) == names
    return db


ann_row = case(test_add, db=schema.product, name="ann", names=["ann"])
cy_row = case(test_add, db=schema.product, name="cy", names=["cy"])
bob_row = case(test_add, db=ann_row.product, name="bob", names=["ann", "bob"])
