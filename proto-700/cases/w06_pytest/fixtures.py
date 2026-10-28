from sketch import fixture, fixture_each
from subject import Client, Clock, Database, Mailer, Settings, Tx


@fixture
def database() -> Database:
    return Database("main")


@fixture
def source() -> Database:
    return Database("source")


@fixture
def target() -> Database:
    return Database("target")


def sqlite() -> Database:
    return Database("sqlite")


def postgres() -> Database:
    return Database("postgres")


@fixture
@fixture_each(sqlite, postgres)
def engine(made: Database) -> Tx[Database]:
    return Tx(made)


@fixture
def db(database: Database) -> Tx[Database]:
    return Tx(database)


@fixture
def src(source: Database) -> Tx[Database]:
    return Tx(source)


@fixture
def dst(target: Database) -> Tx[Database]:
    return Tx(target)


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
