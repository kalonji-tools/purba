from sketch import fixture, use_any
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


@fixture
def big_source() -> Database:
    big = Database("big source")
    big.table.extend(["a", "b", "c"])
    return big


@fixture
def sqlite() -> Database:
    return Database("sqlite")


@fixture
def postgres() -> Database:
    return Database("postgres")


@fixture
def tx[D: Database](db: D) -> Tx[D]:
    return Tx(db)


@fixture
def settings() -> Settings:
    return Settings("test")


@fixture
def client(settings: Settings = use_any(settings)) -> Client:
    return Client(settings)


@fixture
def clock() -> Clock:
    return Clock(0)


@fixture
def mailer() -> Mailer:
    return Mailer()
