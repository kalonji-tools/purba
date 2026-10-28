"""Runtime facts each way rests on. usage: python probe.py"""

import platform
import sqlite3
import typing
from typing import Annotated, NewType

from subject import Database


def show(label, thunk):
    try:
        value = thunk()
    except Exception as error:  # noqa: BLE001
        value = f"{type(error).__name__}: {error}"
    print(f"{platform.python_version():8} {label:62} {value}")


A1 = NewType("SourceDb", Database)
A2 = NewType("SourceDb", Database)
show("NewType: two of one name are one key", lambda: A1 == A2)
show("NewType: the value's type at runtime", lambda: type(A1(Database())).__name__)
show("NewType: isinstance(x, SourceDb)", lambda: isinstance(Database(), A1))

B1 = Annotated[Database, "source"]
B2 = Annotated[Database, "source"]
show("Annotated: two spellings of one marker are one key", lambda: B1 == B2 and hash(B1) == hash(B2))
show("Annotated: nested markers flatten", lambda: Annotated[B1, "x"] == Annotated[Database, "source", "x"])


def t(src: B1, dst: Annotated[Database, "target"]) -> None: ...


show("Annotated: get_type_hints() WITHOUT include_extras", lambda: typing.get_type_hints(t))

Db = Database
show("plain alias `Db = Database` is the type itself", lambda: Db is Database)
ns: dict = {}
exec("type S = Database\ntype T = Database\ntype S2 = Database", {"Database": Database}, ns)
show("`type` alias: S == Database", lambda: ns["S"] == Database)
show("`type` alias: two aliases of one value are one key", lambda: ns["S"] == ns["S2"])

show("subclass: sqlite3.Connection, through factory=", lambda: type(sqlite3.connect(":memory:", factory=type("SourceConn", (sqlite3.Connection,), {}))).__name__)
show("subclass: a final type (bool)", lambda: type("Flag", (bool,), {}))
