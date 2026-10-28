from typing import NewType

from sketch import fixture, fixture_each
from subject import Client, Clock, Database, Mailer, Settings, Tx


SourceDb = NewType("SourceDb", Database)
TargetDb = NewType("TargetDb", Database)
EngineDb = NewType("EngineDb", Database)


type DbTx = Tx[Database]
type SourceTx = Tx[SourceDb]
type TargetTx = Tx[TargetDb]
type EngineTx = Tx[EngineDb]


@fixture
def database() -> Database:
    return Database("main")


@fixture
def source() -> SourceDb:
    return SourceDb(Database("source"))


@fixture
def target() -> TargetDb:
    return TargetDb(Database("target"))


def sqlite() -> Database:
    return Database("sqlite")


def postgres() -> Database:
    return Database("postgres")


@fixture
@fixture_each(sqlite, postgres)
def engine(made: Database) -> EngineDb:
    return EngineDb(made)


@fixture
def tx[D: Database](db: D) -> Tx[D]:
    return Tx(db)


@fixture
def settings() -> Settings:
    return Settings("test")


@fixture
def client(settings: Settings) -> Client:
    return Client(settings)


@fixture
def clock() -> Clock:
    return Clock(0)


@fixture
def mailer() -> Mailer:
    return Mailer()
