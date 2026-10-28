"""A `type` alias, to the type checkers: is it a new type, or the type itself?"""

class Database: ...

type SourceDb = Database
type TargetDb = Database


def wants_source(db: SourceDb) -> None: ...


def give(db: TargetDb) -> Database:
    wants_source(db)          # a TargetDb where a SourceDb is asked
    wants_source(Database())  # a plain Database where a SourceDb is asked
    return db
